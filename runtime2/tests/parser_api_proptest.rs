#![allow(clippy::needless_range_loop)]

use proptest::prelude::*;
use std::time::Duration;

use adze_runtime::language::SymbolMetadata;
use adze_runtime::test_helpers::{multi_symbol_test_language, stub_language};
use adze_runtime::{Language, Parser, Tree};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_duration() -> impl Strategy<Value = Duration> {
    (0u64..10_000_000).prop_map(Duration::from_micros)
}

fn arb_symbol_metadata() -> impl Strategy<Value = SymbolMetadata> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(is_terminal, is_visible, is_supertype)| SymbolMetadata {
            is_terminal,
            is_visible,
            is_supertype,
        },
    )
}

fn arb_input_bytes() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..256)
}

fn arb_utf8_string() -> impl Strategy<Value = String> {
    ".{0,128}"
}

// ---------------------------------------------------------------------------
// 1 – Parser creation
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn new_and_default_parsers_have_no_state(_ in 0..1u8) {
        let p1 = Parser::new();
        let p2 = Parser::default();
        prop_assert!(p1.language().is_none());
        prop_assert!(p2.language().is_none());
        prop_assert!(p1.timeout().is_none());
        prop_assert!(p2.timeout().is_none());
    }
}

// ---------------------------------------------------------------------------
// 2 – Timeout roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn timeout_roundtrip(dur in arb_duration()) {
        let mut parser = Parser::new();
        parser.set_timeout(dur);
        prop_assert_eq!(parser.timeout(), Some(dur));
    }

    #[test]
    fn timeout_last_set_wins(
        dur1 in arb_duration(),
        dur2 in arb_duration(),
    ) {
        let mut parser = Parser::new();
        parser.set_timeout(dur1);
        parser.set_timeout(dur2);
        prop_assert_eq!(parser.timeout(), Some(dur2));
    }

    #[test]
    fn timeout_from_various_units(micros in 0u64..1_000_000_000) {
        let mut parser = Parser::new();
        let dur = Duration::from_micros(micros);
        parser.set_timeout(dur);
        prop_assert_eq!(parser.timeout().unwrap().as_micros(), dur.as_micros());
    }
}

// ---------------------------------------------------------------------------
// 3 – Language setting and accessor
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn set_language_succeeds_with_stub(_ in 0..1u8) {
        let mut parser = Parser::new();
        let lang = stub_language();
        prop_assert!(parser.set_language(lang).is_ok());
        prop_assert!(parser.language().is_some());
    }

    #[test]
    fn language_symbol_count_matches_after_set(count in 1usize..20) {
        let mut parser = Parser::new();
        let lang = multi_symbol_test_language(count);
        parser.set_language(lang).unwrap();
        let lang_ref = parser.language().unwrap();
        prop_assert_eq!(lang_ref.symbol_count as usize, count);
        prop_assert_eq!(lang_ref.symbol_names.len(), count);
    }

    #[test]
    fn set_language_replaces_previous(_ in 0..5u8) {
        let mut parser = Parser::new();
        let lang1 = multi_symbol_test_language(2);
        let lang2 = multi_symbol_test_language(5);
        parser.set_language(lang1).unwrap();
        prop_assert_eq!(parser.language().unwrap().symbol_count, 2);
        parser.set_language(lang2).unwrap();
        prop_assert_eq!(parser.language().unwrap().symbol_count, 5);
    }
}

// ---------------------------------------------------------------------------
// 4 – Parse without language returns error
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn parse_bytes_without_language_errors(input in arb_input_bytes()) {
        let mut parser = Parser::new();
        let err = parser.parse(&input, None).unwrap_err();
        prop_assert!(err.to_string().contains("no language"));
    }

    #[test]
    fn parse_utf8_without_language_errors(input in arb_utf8_string()) {
        let mut parser = Parser::new();
        let err = parser.parse_utf8(&input, None).unwrap_err();
        prop_assert!(err.to_string().contains("no language"));
    }

    #[test]
    fn parse_empty_without_language_errors(_ in 0..1u8) {
        let mut parser = Parser::new();
        prop_assert!(parser.parse(b"", None).is_err());
        prop_assert!(parser.parse_utf8("", None).is_err());
    }
}

// ---------------------------------------------------------------------------
// 5 – Reset behaviour
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn reset_preserves_language(_ in 0..1u8) {
        let mut parser = Parser::new();
        parser.set_language(stub_language()).unwrap();
        parser.reset();
        prop_assert!(parser.language().is_some());
    }

    #[test]
    fn reset_preserves_timeout(dur in arb_duration()) {
        let mut parser = Parser::new();
        parser.set_timeout(dur);
        parser.reset();
        prop_assert_eq!(parser.timeout(), Some(dur));
    }

    #[test]
    fn multiple_resets_are_idempotent(_ in 0..1u8) {
        let mut parser = Parser::new();
        parser.set_language(stub_language()).unwrap();
        parser.set_timeout(Duration::from_millis(100));
        for _ in 0..10 {
            parser.reset();
        }
        prop_assert!(parser.language().is_some());
        prop_assert_eq!(parser.timeout(), Some(Duration::from_millis(100)));
    }
}

// ---------------------------------------------------------------------------
// 6 – Language builder
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn builder_version_is_stored(version in any::<u32>()) {
        let lang = make_language_with_version(version);
        prop_assert_eq!(lang.version, version);
    }

    #[test]
    fn builder_field_count_matches(field_count in 0usize..10) {
        let field_names: Vec<String> = (0..field_count).map(|i| format!("field_{i}")).collect();
        let lang = make_language_with_fields(field_names.clone());
        prop_assert_eq!(lang.field_count as usize, field_count);
        prop_assert_eq!(lang.field_names.len(), field_count);
    }

    #[test]
    fn builder_max_alias_sequence_length(len in any::<u32>()) {
        let table = leak_empty_parse_table();
        let lang = Language::builder()
            .parse_table(table)
            .max_alias_sequence_length(len)
            .symbol_metadata(vec![SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            }])
            .tokenizer(|_| Box::new(std::iter::empty()))
            .build()
            .unwrap();
        prop_assert_eq!(lang.max_alias_sequence_length, len);
    }

    #[test]
    fn builder_without_parse_table_fails(_ in 0..1u8) {
        let result = Language::builder()
            .symbol_metadata(vec![SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            }])
            .build();
        prop_assert!(result.is_err());
        prop_assert_eq!(result.unwrap_err(), "missing parse table");
    }

    #[test]
    fn builder_without_metadata_fails(_ in 0..1u8) {
        let table = leak_empty_parse_table();
        let result = Language::builder().parse_table(table).build();
        prop_assert!(result.is_err());
        prop_assert_eq!(result.unwrap_err(), "missing symbol metadata");
    }
}

// ---------------------------------------------------------------------------
// 7 – Language symbol accessors
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn symbol_name_in_bounds(count in 1usize..20, idx_offset in 0usize..20) {
        let lang = multi_symbol_test_language(count);
        let idx = idx_offset % count;
        let name = lang.symbol_name(idx as u16);
        prop_assert!(name.is_some());
        prop_assert_eq!(name.unwrap(), &format!("symbol_{idx}"));
    }

    #[test]
    fn symbol_name_out_of_bounds(count in 1usize..10) {
        let lang = multi_symbol_test_language(count);
        prop_assert!(lang.symbol_name((count + 100) as u16).is_none());
    }

    #[test]
    fn is_terminal_and_visible_for_valid_index(count in 1usize..20) {
        let lang = multi_symbol_test_language(count);
        for i in 0..count {
            prop_assert!(lang.is_terminal(i as u16));
            prop_assert!(lang.is_visible(i as u16));
        }
    }

    #[test]
    fn is_terminal_and_visible_out_of_bounds(count in 1usize..10) {
        let lang = multi_symbol_test_language(count);
        prop_assert!(!lang.is_terminal((count + 100) as u16));
        prop_assert!(!lang.is_visible((count + 100) as u16));
    }
}

// ---------------------------------------------------------------------------
// 8 – Language clone
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn language_clone_preserves_all_fields(count in 1usize..10) {
        let lang = multi_symbol_test_language(count);
        let cloned = lang.clone();
        prop_assert_eq!(cloned.version, lang.version);
        prop_assert_eq!(cloned.symbol_count, lang.symbol_count);
        prop_assert_eq!(cloned.field_count, lang.field_count);
        prop_assert_eq!(cloned.max_alias_sequence_length, lang.max_alias_sequence_length);
        prop_assert_eq!(&cloned.symbol_names, &lang.symbol_names);
        prop_assert_eq!(&cloned.field_names, &lang.field_names);
    }
}

// ---------------------------------------------------------------------------
// 9 – Debug impls
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn parser_debug_no_panic_any_state(_ in 0..1u8) {
        let parser = Parser::new();
        let dbg = format!("{:?}", parser);
        prop_assert!(dbg.contains("Parser"));

        let mut parser2 = Parser::new();
        parser2.set_language(stub_language()).unwrap();
        let dbg2 = format!("{:?}", parser2);
        prop_assert!(dbg2.contains("Parser"));
    }

    #[test]
    fn language_debug_no_panic(count in 1usize..10) {
        let lang = multi_symbol_test_language(count);
        let dbg = format!("{:?}", lang);
        prop_assert!(dbg.contains("Language"));
        prop_assert!(dbg.contains("symbol_count"));
    }

    #[test]
    fn tree_stub_debug_no_panic(_ in 0..1u8) {
        let tree = Tree::new_stub();
        let dbg = format!("{:?}", tree);
        prop_assert!(dbg.contains("Tree"));
    }
}

// ---------------------------------------------------------------------------
// 10 – Tree::new_stub
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn stub_tree_root_node_defaults(_ in 0..1u8) {
        let tree = Tree::new_stub();
        let root = tree.root_node();
        prop_assert_eq!(root.kind(), "unknown");
        prop_assert_eq!(root.kind_id(), 0);
        prop_assert_eq!(root.start_byte(), 0);
        prop_assert_eq!(root.end_byte(), 0);
        prop_assert_eq!(root.child_count(), 0);
        prop_assert_eq!(tree.root_kind(), 0);
    }

    #[test]
    fn stub_tree_has_no_language_or_source(_ in 0..1u8) {
        let tree = Tree::new_stub();
        prop_assert!(tree.language().is_none());
        prop_assert!(tree.source_bytes().is_none());
    }

    #[test]
    fn stub_tree_clone_is_independent(_ in 0..1u8) {
        let tree = Tree::new_stub();
        let cloned = tree.clone();
        prop_assert_eq!(tree.root_node().kind_id(), cloned.root_node().kind_id());
        prop_assert_eq!(tree.root_node().start_byte(), cloned.root_node().start_byte());
    }
}

// ---------------------------------------------------------------------------
// 11 – Multiple independent parsers
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn independent_parsers_have_separate_state(
        dur1 in arb_duration(),
        dur2 in arb_duration(),
    ) {
        let mut p1 = Parser::new();
        let mut p2 = Parser::new();
        p1.set_timeout(dur1);
        p2.set_timeout(dur2);
        p1.set_language(stub_language()).unwrap();
        // p2 has no language
        prop_assert!(p1.language().is_some());
        prop_assert!(p2.language().is_none());
        prop_assert_eq!(p1.timeout(), Some(dur1));
        prop_assert_eq!(p2.timeout(), Some(dur2));
    }
}

// ---------------------------------------------------------------------------
// 12 – Language field_name accessor
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn field_name_in_bounds(field_count in 1usize..10) {
        let field_names: Vec<String> = (0..field_count).map(|i| format!("f_{i}")).collect();
        let lang = make_language_with_fields(field_names.clone());
        for i in 0..field_count {
            let name = lang.field_name(i as u16);
            prop_assert!(name.is_some());
            prop_assert_eq!(name.unwrap(), &format!("f_{i}"));
        }
    }

    #[test]
    fn field_name_out_of_bounds(_ in 0..1u8) {
        let lang = stub_language();
        prop_assert!(lang.field_name(0).is_none());
        prop_assert!(lang.field_name(999).is_none());
    }
}

// ---------------------------------------------------------------------------
// 13 – Symbol metadata properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn symbol_metadata_fields_roundtrip(meta in arb_symbol_metadata()) {
        let meta2 = SymbolMetadata {
            is_terminal: meta.is_terminal,
            is_visible: meta.is_visible,
            is_supertype: meta.is_supertype,
        };
        prop_assert_eq!(meta.is_terminal, meta2.is_terminal);
        prop_assert_eq!(meta.is_visible, meta2.is_visible);
        prop_assert_eq!(meta.is_supertype, meta2.is_supertype);
    }

    #[test]
    fn symbol_metadata_debug_contains_fields(meta in arb_symbol_metadata()) {
        let dbg = format!("{:?}", meta);
        prop_assert!(dbg.contains("SymbolMetadata"));
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn leak_empty_parse_table() -> &'static adze_glr_core::ParseTable {
    Box::leak(Box::new(adze_glr_core::ParseTable::default()))
}

fn make_language_with_version(version: u32) -> Language {
    let table = leak_empty_parse_table();
    Language::builder()
        .version(version)
        .parse_table(table)
        .symbol_names(vec!["sym".into()])
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }])
        .tokenizer(|_| Box::new(std::iter::empty()))
        .build()
        .unwrap()
}

// ---------------------------------------------------------------------------
// 14 – Tree::new_for_testing
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn new_for_testing_preserves_symbol_and_range(
        sym in 0u32..100,
        start in 0usize..1000,
        len in 0usize..500,
    ) {
        let end = start + len;
        let tree = Tree::new_for_testing(sym, start, end, vec![]);
        prop_assert_eq!(tree.root_kind(), sym);
        prop_assert_eq!(tree.root_node().start_byte(), start);
        prop_assert_eq!(tree.root_node().end_byte(), end);
        prop_assert_eq!(tree.root_node().child_count(), 0);
    }

    #[test]
    fn new_for_testing_with_children(child_count in 1usize..8) {
        let children: Vec<Tree> = (0..child_count)
            .map(|i| Tree::new_for_testing(i as u32 + 1, i * 10, (i + 1) * 10, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, child_count * 10, children);
        prop_assert_eq!(tree.root_node().child_count(), child_count);
    }
}

// ---------------------------------------------------------------------------
// 15 – Node byte_range and kind_id
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn node_byte_range_matches_start_end(
        start in 0usize..1000,
        len in 0usize..500,
    ) {
        let end = start + len;
        let tree = Tree::new_for_testing(42, start, end, vec![]);
        let root = tree.root_node();
        prop_assert_eq!(root.byte_range(), start..end);
        prop_assert_eq!(root.start_byte(), start);
        prop_assert_eq!(root.end_byte(), end);
    }

    #[test]
    fn node_kind_id_matches_symbol(sym in 0u32..65535) {
        let tree = Tree::new_for_testing(sym, 0, 0, vec![]);
        prop_assert_eq!(tree.root_node().kind_id(), sym as u16);
    }
}

// ---------------------------------------------------------------------------
// 16 – Node child access
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn node_child_in_bounds(child_count in 1usize..10) {
        let children: Vec<Tree> = (0..child_count)
            .map(|i| Tree::new_for_testing(i as u32 + 1, i, i + 1, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, child_count, children);
        let root = tree.root_node();
        for i in 0..child_count {
            let child = root.child(i);
            prop_assert!(child.is_some());
            prop_assert_eq!(child.unwrap().kind_id(), (i as u16) + 1);
        }
    }

    #[test]
    fn node_child_out_of_bounds_returns_none(child_count in 0usize..5) {
        let children: Vec<Tree> = (0..child_count)
            .map(|i| Tree::new_for_testing(i as u32, 0, 0, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, 0, children);
        prop_assert!(tree.root_node().child(child_count).is_none());
        prop_assert!(tree.root_node().child(child_count + 100).is_none());
    }

    #[test]
    fn named_child_same_as_child(idx in 0usize..5) {
        let children: Vec<Tree> = (0..5)
            .map(|i| Tree::new_for_testing(i as u32, 0, 0, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, 0, children);
        let root = tree.root_node();
        let c = root.child(idx);
        let nc = root.named_child(idx);
        prop_assert_eq!(c.map(|n| n.kind_id()), nc.map(|n| n.kind_id()));
    }

    #[test]
    fn named_child_count_equals_child_count(count in 0usize..8) {
        let children: Vec<Tree> = (0..count)
            .map(|i| Tree::new_for_testing(i as u32, 0, 0, vec![]))
            .collect();
        let tree = Tree::new_for_testing(0, 0, 0, children);
        prop_assert_eq!(tree.root_node().named_child_count(), tree.root_node().child_count());
    }
}

// ---------------------------------------------------------------------------
// 17 – Node status flags
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn node_is_named_always_true(_ in 0..1u8) {
        let tree = Tree::new_for_testing(0, 0, 10, vec![]);
        prop_assert!(tree.root_node().is_named());
    }

    #[test]
    fn node_is_missing_always_false(_ in 0..1u8) {
        let tree = Tree::new_for_testing(0, 0, 10, vec![]);
        prop_assert!(!tree.root_node().is_missing());
    }

    #[test]
    fn node_is_error_always_false(_ in 0..1u8) {
        let tree = Tree::new_for_testing(0, 0, 10, vec![]);
        prop_assert!(!tree.root_node().is_error());
    }
}

// ---------------------------------------------------------------------------
// 18 – Node navigation stubs (parent/sibling return None)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn node_parent_returns_none(_ in 0..1u8) {
        let tree = Tree::new_for_testing(0, 0, 10, vec![
            Tree::new_for_testing(1, 0, 5, vec![]),
        ]);
        prop_assert!(tree.root_node().parent().is_none());
        prop_assert!(tree.root_node().child(0).unwrap().parent().is_none());
    }

    #[test]
    fn node_sibling_accessors_return_none(_ in 0..1u8) {
        let tree = Tree::new_for_testing(0, 0, 10, vec![
            Tree::new_for_testing(1, 0, 5, vec![]),
            Tree::new_for_testing(2, 5, 10, vec![]),
        ]);
        let child = tree.root_node().child(0).unwrap();
        prop_assert!(child.next_sibling().is_none());
        prop_assert!(child.prev_sibling().is_none());
        prop_assert!(child.next_named_sibling().is_none());
        prop_assert!(child.prev_named_sibling().is_none());
    }

    #[test]
    fn node_child_by_field_name_returns_none(_ in 0..1u8) {
        let tree = Tree::new_for_testing(0, 0, 10, vec![]);
        prop_assert!(tree.root_node().child_by_field_name("anything").is_none());
    }
}

// ---------------------------------------------------------------------------
// 19 – Node.utf8_text
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn utf8_text_extracts_correct_slice(s in "[a-z]{1,50}") {
        let bytes = s.as_bytes();
        let tree = Tree::new_for_testing(0, 0, bytes.len(), vec![]);
        let text = tree.root_node().utf8_text(bytes).unwrap();
        prop_assert_eq!(text, s.as_str());
    }

    #[test]
    fn utf8_text_partial_range(
        prefix in "[a-z]{1,10}",
        middle in "[a-z]{1,10}",
        suffix in "[a-z]{1,10}",
    ) {
        let full = format!("{}{}{}", prefix, middle, suffix);
        let start = prefix.len();
        let end = prefix.len() + middle.len();
        let tree = Tree::new_for_testing(0, start, end, vec![]);
        let text = tree.root_node().utf8_text(full.as_bytes()).unwrap();
        prop_assert_eq!(text, middle.as_str());
    }
}

// ---------------------------------------------------------------------------
// 20 – Node.start_position / end_position (stub)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn start_end_position_are_zero(_ in 0..1u8) {
        use adze_runtime::Point;
        let tree = Tree::new_for_testing(0, 10, 20, vec![]);
        let root = tree.root_node();
        prop_assert_eq!(root.start_position(), Point::new(0, 0));
        prop_assert_eq!(root.end_position(), Point::new(0, 0));
    }
}

// ---------------------------------------------------------------------------
// 21 – Point ordering and Display
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn point_new_roundtrip(row in 0usize..1000, col in 0usize..1000) {
        use adze_runtime::Point;
        let p = Point::new(row, col);
        prop_assert_eq!(p.row, row);
        prop_assert_eq!(p.column, col);
    }

    #[test]
    fn point_display_is_one_indexed(row in 0usize..100, col in 0usize..100) {
        use adze_runtime::Point;
        let p = Point::new(row, col);
        let display = format!("{}", p);
        prop_assert_eq!(display, format!("{}:{}", row + 1, col + 1));
    }

    #[test]
    fn point_ordering_row_then_col(
        r1 in 0usize..100, c1 in 0usize..100,
        r2 in 0usize..100, c2 in 0usize..100,
    ) {
        use adze_runtime::Point;
        let p1 = Point::new(r1, c1);
        let p2 = Point::new(r2, c2);
        let expected = (r1, c1).cmp(&(r2, c2));
        prop_assert_eq!(p1.cmp(&p2), expected);
    }
}

// ---------------------------------------------------------------------------
// 22 – ParseError constructors and Display
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn parse_error_no_language_display(_ in 0..1u8) {
        use adze_runtime::ParseError;
        let err = ParseError::no_language();
        let msg = err.to_string();
        prop_assert!(msg.contains("no language"));
    }

    #[test]
    fn parse_error_timeout_display(_ in 0..1u8) {
        use adze_runtime::ParseError;
        let err = ParseError::timeout();
        let msg = err.to_string();
        prop_assert!(msg.contains("timeout"));
    }

    #[test]
    fn parse_error_with_msg_display(s in "[a-z ]{1,50}") {
        use adze_runtime::ParseError;
        let err = ParseError::with_msg(&s);
        prop_assert_eq!(err.to_string(), s);
    }

    #[test]
    fn parse_error_syntax_error_has_location(
        offset in 0usize..1000,
        line in 1usize..100,
        col in 1usize..100,
    ) {
        use adze_runtime::error::{ErrorLocation, ParseError};
        let loc = ErrorLocation { byte_offset: offset, line, column: col };
        let err = ParseError::syntax_error("bad token", loc.clone());
        prop_assert!(err.location.is_some());
        prop_assert_eq!(err.location.unwrap(), loc);
    }

    #[test]
    fn parse_error_with_location_attaches(
        offset in 0usize..1000,
        line in 1usize..100,
        col in 1usize..100,
    ) {
        use adze_runtime::error::{ErrorLocation, ParseError};
        let loc = ErrorLocation { byte_offset: offset, line, column: col };
        let err = ParseError::no_language().with_location(loc.clone());
        prop_assert_eq!(err.location.unwrap(), loc);
    }
}

// ---------------------------------------------------------------------------
// 23 – ErrorLocation Display
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn error_location_display(line in 1usize..1000, col in 1usize..1000) {
        use adze_runtime::error::ErrorLocation;
        let loc = ErrorLocation { byte_offset: 0, line, column: col };
        prop_assert_eq!(format!("{}", loc), format!("{}:{}", line, col));
    }
}

// ---------------------------------------------------------------------------
// 24 – Language.symbol_for_name
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn symbol_for_name_finds_existing(count in 2usize..10) {
        let lang = multi_symbol_test_language(count);
        // All symbols in multi_symbol_test_language are visible
        for i in 0..count {
            let result = lang.symbol_for_name(&format!("symbol_{}", i), true);
            prop_assert_eq!(result, Some(i as u16));
        }
    }

    #[test]
    fn symbol_for_name_returns_none_for_missing(_ in 0..1u8) {
        let lang = multi_symbol_test_language(3);
        prop_assert!(lang.symbol_for_name("nonexistent", true).is_none());
    }

    #[test]
    fn symbol_for_name_respects_is_named(count in 2usize..10) {
        let lang = multi_symbol_test_language(count);
        // All metadata is visible=true, so is_named=false should return None
        for i in 0..count {
            let result = lang.symbol_for_name(&format!("symbol_{}", i), false);
            prop_assert!(result.is_none());
        }
    }
}

// ---------------------------------------------------------------------------
// 25 – Parser set_language validation
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn set_language_rejects_empty_metadata(_ in 0..1u8) {
        let table = leak_empty_parse_table();
        let lang = Language {
            version: 1,
            symbol_count: 0,
            field_count: 0,
            max_alias_sequence_length: 0,
            parse_table: Some(table),
            tokenize: Some(Box::new(|_: &[u8]| -> Box<dyn Iterator<Item = adze_runtime::Token>> {
                Box::new(std::iter::empty())
            })),
            symbol_names: vec![],
            symbol_metadata: vec![],
            field_names: vec![],
            #[cfg(feature = "external_scanners")]
            external_scanner: None,
        };
        let mut parser = Parser::new();
        let result = parser.set_language(lang);
        prop_assert!(result.is_err());
        prop_assert!(result.unwrap_err().to_string().contains("no symbol metadata"));
    }
}

// ---------------------------------------------------------------------------
// 26 – Node Debug impl
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn node_debug_contains_kind_and_range(
        start in 0usize..100,
        len in 0usize..100,
    ) {
        let end = start + len;
        let tree = Tree::new_for_testing(0, start, end, vec![]);
        let dbg = format!("{:?}", tree.root_node());
        prop_assert!(dbg.contains("Node"));
        prop_assert!(dbg.contains("kind"));
        prop_assert!(dbg.contains("range"));
    }
}

fn make_language_with_fields(field_names: Vec<String>) -> Language {
    let table = leak_empty_parse_table();
    Language::builder()
        .parse_table(table)
        .symbol_names(vec!["sym".into()])
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }])
        .field_names(field_names)
        .tokenizer(|_| Box::new(std::iter::empty()))
        .build()
        .unwrap()
}
