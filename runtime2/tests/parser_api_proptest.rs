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
