#![allow(clippy::needless_range_loop)]

//! Property-based tests for LanguageBuilder in adze-runtime.

use proptest::prelude::*;

use adze_runtime::Token;
use adze_runtime::language::{Language, SymbolMetadata};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_symbol_metadata() -> impl Strategy<Value = SymbolMetadata> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(|(t, v, s)| SymbolMetadata {
        is_terminal: t,
        is_visible: v,
        is_supertype: s,
    })
}

fn arb_symbol_metadata_vec(min: usize, max: usize) -> impl Strategy<Value = Vec<SymbolMetadata>> {
    proptest::collection::vec(arb_symbol_metadata(), min..=max)
}

fn arb_version() -> impl Strategy<Value = u32> {
    prop_oneof![Just(0u32), Just(1), Just(14), Just(15), 0..=1000u32,]
}

fn arb_alias_len() -> impl Strategy<Value = u32> {
    prop_oneof![Just(0u32), 0..=50u32,]
}

fn leak_table() -> &'static adze_glr_core::ParseTable {
    Box::leak(Box::new(adze_glr_core::ParseTable::default()))
}

fn build_minimal(meta: Vec<SymbolMetadata>) -> Language {
    Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(meta)
        .build()
        .unwrap()
}

fn build_full(
    version: u32,
    alias_len: u32,
    meta: Vec<SymbolMetadata>,
    names: Vec<String>,
    fields: Vec<String>,
) -> Language {
    Language::builder()
        .version(version)
        .max_alias_sequence_length(alias_len)
        .parse_table(leak_table())
        .symbol_names(names)
        .symbol_metadata(meta)
        .field_names(fields)
        .build()
        .unwrap()
}

// ---------------------------------------------------------------------------
// 1 – Builder creates Language successfully
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn builder_creates_language_with_arbitrary_metadata(
        meta in arb_symbol_metadata_vec(1, 30)
    ) {
        let result = Language::builder()
            .parse_table(leak_table())
            .symbol_metadata(meta)
            .build();
        prop_assert!(result.is_ok());
    }

    #[test]
    fn builder_creates_language_with_version(version in arb_version()) {
        let lang = Language::builder()
            .version(version)
            .parse_table(leak_table())
            .symbol_metadata(vec![SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            }])
            .build()
            .unwrap();
        prop_assert_eq!(lang.version, version);
    }

    #[test]
    fn builder_creates_language_with_alias_len(alias_len in arb_alias_len()) {
        let lang = Language::builder()
            .max_alias_sequence_length(alias_len)
            .parse_table(leak_table())
            .symbol_metadata(vec![SymbolMetadata {
                is_terminal: true,
                is_visible: false,
                is_supertype: false,
            }])
            .build()
            .unwrap();
        prop_assert_eq!(lang.max_alias_sequence_length, alias_len);
    }
}

// ---------------------------------------------------------------------------
// 2 – Builder requires parse_table
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn builder_without_parse_table_fails(meta in arb_symbol_metadata_vec(0, 10)) {
        let result = Language::builder()
            .symbol_metadata(meta)
            .build();
        prop_assert!(result.is_err());
        prop_assert_eq!(result.unwrap_err(), "missing parse table");
    }

    #[test]
    fn builder_without_parse_table_fails_even_with_all_optionals(
        version in arb_version(),
        alias_len in arb_alias_len(),
        meta in arb_symbol_metadata_vec(1, 5),
    ) {
        let names: Vec<String> = (0..meta.len()).map(|i| format!("s{i}")).collect();
        let result = Language::builder()
            .version(version)
            .max_alias_sequence_length(alias_len)
            .symbol_names(names)
            .symbol_metadata(meta)
            .field_names(vec!["f".into()])
            .build();
        prop_assert!(result.is_err());
        prop_assert_eq!(result.unwrap_err(), "missing parse table");
    }
}

// ---------------------------------------------------------------------------
// 3 – Builder requires symbol_metadata
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn builder_without_metadata_fails(version in arb_version()) {
        let result = Language::builder()
            .version(version)
            .parse_table(leak_table())
            .build();
        prop_assert!(result.is_err());
        prop_assert_eq!(result.unwrap_err(), "missing symbol metadata");
    }

    #[test]
    fn builder_without_metadata_fails_with_names_and_fields(
        n in 1usize..10,
    ) {
        let names: Vec<String> = (0..n).map(|i| format!("sym_{i}")).collect();
        let fields: Vec<String> = (0..n).map(|i| format!("field_{i}")).collect();
        let result = Language::builder()
            .parse_table(leak_table())
            .symbol_names(names)
            .field_names(fields)
            .build();
        prop_assert!(result.is_err());
        prop_assert_eq!(result.unwrap_err(), "missing symbol metadata");
    }
}

// ---------------------------------------------------------------------------
// 4 – Builder with custom tokenizer
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn builder_with_empty_tokenizer_succeeds(meta in arb_symbol_metadata_vec(1, 10)) {
        let lang = Language::builder()
            .parse_table(leak_table())
            .symbol_metadata(meta)
            .tokenizer(|_| Box::new(std::iter::empty()))
            .build()
            .unwrap();
        prop_assert!(lang.tokenize.is_some());
    }

    #[test]
    fn tokenizer_output_length_matches_input_byte_count(
        input in proptest::collection::vec(any::<u8>(), 0..50)
    ) {
        let lang = Language::builder()
            .parse_table(leak_table())
            .symbol_metadata(vec![SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            }])
            .tokenizer(|bytes| {
                Box::new(bytes.iter().enumerate().map(|(i, &b)| Token {
                    kind: b as u32,
                    start: i as u32,
                    end: (i + 1) as u32,
                }))
            })
            .build()
            .unwrap();
        let tok = lang.tokenize.as_ref().unwrap();
        let tokens: Vec<_> = tok(&input).collect();
        prop_assert_eq!(tokens.len(), input.len());
    }

    #[test]
    fn tokenizer_spans_are_contiguous(
        input in proptest::collection::vec(any::<u8>(), 1..30)
    ) {
        let lang = Language::builder()
            .parse_table(leak_table())
            .symbol_metadata(vec![SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            }])
            .tokenizer(|bytes| {
                Box::new(bytes.iter().enumerate().map(|(i, &b)| Token {
                    kind: b as u32,
                    start: i as u32,
                    end: (i + 1) as u32,
                }))
            })
            .build()
            .unwrap();
        let tok = lang.tokenize.as_ref().unwrap();
        let tokens: Vec<_> = tok(&input).collect();
        for i in 1..tokens.len() {
            prop_assert_eq!(tokens[i].start, tokens[i - 1].end);
        }
    }
}

// ---------------------------------------------------------------------------
// 5 – Builder error when missing fields
// ---------------------------------------------------------------------------

#[test]
fn empty_builder_fails_with_parse_table_error() {
    let result = Language::builder().build();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "missing parse table");
}

#[test]
fn builder_only_version_fails() {
    let result = Language::builder().version(15).build();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "missing parse table");
}

#[test]
fn builder_only_alias_fails() {
    let result = Language::builder().max_alias_sequence_length(3).build();
    assert!(result.is_err());
}

proptest! {
    #[test]
    fn missing_both_required_always_reports_parse_table_first(
        version in arb_version(),
        alias_len in arb_alias_len(),
    ) {
        let result = Language::builder()
            .version(version)
            .max_alias_sequence_length(alias_len)
            .build();
        prop_assert_eq!(result.unwrap_err(), "missing parse table");
    }
}

// ---------------------------------------------------------------------------
// 6 – Builder determinism
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn two_builds_with_same_inputs_produce_equal_languages(
        version in arb_version(),
        alias_len in arb_alias_len(),
        meta in arb_symbol_metadata_vec(1, 20),
    ) {
        let names: Vec<String> = (0..meta.len()).map(|i| format!("sym_{i}")).collect();
        let fields = vec!["f0".to_string(), "f1".to_string()];

        let l1 = build_full(version, alias_len, meta.clone(), names.clone(), fields.clone());
        let l2 = build_full(version, alias_len, meta.clone(), names.clone(), fields.clone());

        prop_assert_eq!(l1.version, l2.version);
        prop_assert_eq!(l1.symbol_count, l2.symbol_count);
        prop_assert_eq!(l1.field_count, l2.field_count);
        prop_assert_eq!(l1.max_alias_sequence_length, l2.max_alias_sequence_length);
        prop_assert_eq!(l1.symbol_names, l2.symbol_names);
        prop_assert_eq!(l1.field_names, l2.field_names);
        for i in 0..meta.len() {
            prop_assert_eq!(l1.symbol_metadata[i].is_terminal, l2.symbol_metadata[i].is_terminal);
            prop_assert_eq!(l1.symbol_metadata[i].is_visible, l2.symbol_metadata[i].is_visible);
            prop_assert_eq!(l1.symbol_metadata[i].is_supertype, l2.symbol_metadata[i].is_supertype);
        }
    }

    #[test]
    fn debug_output_is_deterministic(meta in arb_symbol_metadata_vec(1, 10)) {
        let l1 = build_minimal(meta.clone());
        let l2 = build_minimal(meta);
        prop_assert_eq!(format!("{l1:?}"), format!("{l2:?}"));
    }

    #[test]
    fn clone_preserves_scalar_fields(
        version in arb_version(),
        alias_len in arb_alias_len(),
        meta in arb_symbol_metadata_vec(1, 10),
    ) {
        let names: Vec<String> = (0..meta.len()).map(|i| format!("n{i}")).collect();
        let lang = build_full(version, alias_len, meta, names, vec!["x".into()]);
        let cloned = lang.clone();
        prop_assert_eq!(cloned.version, lang.version);
        prop_assert_eq!(cloned.symbol_count, lang.symbol_count);
        prop_assert_eq!(cloned.field_count, lang.field_count);
        prop_assert_eq!(cloned.max_alias_sequence_length, lang.max_alias_sequence_length);
        prop_assert_eq!(cloned.symbol_names, lang.symbol_names);
        prop_assert_eq!(cloned.field_names, lang.field_names);
    }
}

// ---------------------------------------------------------------------------
// 7 – Builder with various metadata counts
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_count_equals_metadata_len(meta in arb_symbol_metadata_vec(0, 50)) {
        let lang = build_minimal(meta.clone());
        prop_assert_eq!(lang.symbol_count as usize, meta.len());
    }

    #[test]
    fn field_count_equals_field_names_len(n in 0usize..30) {
        let fields: Vec<String> = (0..n).map(|i| format!("field_{i}")).collect();
        let lang = Language::builder()
            .parse_table(leak_table())
            .symbol_metadata(vec![SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            }])
            .field_names(fields)
            .build()
            .unwrap();
        prop_assert_eq!(lang.field_count as usize, n);
    }

    #[test]
    fn metadata_flags_are_preserved(meta in arb_symbol_metadata_vec(1, 40)) {
        let lang = build_minimal(meta.clone());
        for i in 0..meta.len() {
            prop_assert_eq!(lang.is_terminal(i as u16), meta[i].is_terminal);
            prop_assert_eq!(lang.is_visible(i as u16), meta[i].is_visible);
            prop_assert_eq!(lang.symbol_metadata[i].is_supertype, meta[i].is_supertype);
        }
    }

    #[test]
    fn symbol_names_default_to_empty_when_omitted(meta in arb_symbol_metadata_vec(1, 20)) {
        let lang = build_minimal(meta.clone());
        for i in 0..meta.len() {
            prop_assert_eq!(lang.symbol_name(i as u16), Some(""));
        }
    }

    #[test]
    fn explicit_symbol_names_are_preserved(n in 1usize..25) {
        let meta = vec![SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }; n];
        let names: Vec<String> = (0..n).map(|i| format!("tok_{i}")).collect();
        let lang = Language::builder()
            .parse_table(leak_table())
            .symbol_names(names.clone())
            .symbol_metadata(meta)
            .build()
            .unwrap();
        for i in 0..n {
            prop_assert_eq!(lang.symbol_name(i as u16), Some(names[i].as_str()));
        }
    }
}

// ---------------------------------------------------------------------------
// 8 – Out-of-bounds queries
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_name_oob_returns_none(meta in arb_symbol_metadata_vec(0, 10)) {
        let lang = build_minimal(meta.clone());
        let oob = meta.len() as u16;
        prop_assert!(lang.symbol_name(oob).is_none());
        prop_assert!(lang.symbol_name(oob.saturating_add(100)).is_none());
    }

    #[test]
    fn field_name_oob_returns_none(n in 0usize..10) {
        let fields: Vec<String> = (0..n).map(|i| format!("f{i}")).collect();
        let lang = Language::builder()
            .parse_table(leak_table())
            .symbol_metadata(vec![SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            }])
            .field_names(fields)
            .build()
            .unwrap();
        prop_assert!(lang.field_name(n as u16).is_none());
        prop_assert!(lang.field_name(u16::MAX).is_none());
    }

    #[test]
    fn is_terminal_oob_returns_false(meta in arb_symbol_metadata_vec(0, 10)) {
        let lang = build_minimal(meta.clone());
        prop_assert!(!lang.is_terminal(meta.len() as u16));
    }

    #[test]
    fn is_visible_oob_returns_false(meta in arb_symbol_metadata_vec(0, 10)) {
        let lang = build_minimal(meta.clone());
        prop_assert!(!lang.is_visible(meta.len() as u16));
    }
}

// ---------------------------------------------------------------------------
// 9 – symbol_for_name with builder
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_for_name_finds_visible_symbol(
        idx in 0usize..10,
        n in 1usize..15,
    ) {
        let idx = idx % n;
        let meta: Vec<SymbolMetadata> = (0..n)
            .map(|i| SymbolMetadata {
                is_terminal: i == idx,
                is_visible: i == idx,
                is_supertype: false,
            })
            .collect();
        let names: Vec<String> = (0..n).map(|i| format!("sym_{i}")).collect();
        let lang = Language::builder()
            .parse_table(leak_table())
            .symbol_names(names)
            .symbol_metadata(meta)
            .build()
            .unwrap();
        let target = format!("sym_{idx}");
        prop_assert_eq!(lang.symbol_for_name(&target, true), Some(idx as u16));
    }
}

// ---------------------------------------------------------------------------
// 10 – Builder chain ordering independence
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn builder_order_does_not_matter(
        version in arb_version(),
        meta in arb_symbol_metadata_vec(1, 5),
    ) {
        let names: Vec<String> = (0..meta.len()).map(|i| format!("s{i}")).collect();

        // Order A: version, parse_table, names, meta
        let la = Language::builder()
            .version(version)
            .parse_table(leak_table())
            .symbol_names(names.clone())
            .symbol_metadata(meta.clone())
            .build()
            .unwrap();

        // Order B: meta, names, parse_table, version
        let lb = Language::builder()
            .symbol_metadata(meta.clone())
            .symbol_names(names.clone())
            .parse_table(leak_table())
            .version(version)
            .build()
            .unwrap();

        prop_assert_eq!(la.version, lb.version);
        prop_assert_eq!(la.symbol_count, lb.symbol_count);
        prop_assert_eq!(la.symbol_names, lb.symbol_names);
        for i in 0..meta.len() {
            prop_assert_eq!(la.symbol_metadata[i].is_terminal, lb.symbol_metadata[i].is_terminal);
        }
    }
}

// ---------------------------------------------------------------------------
// 11 – Clone loses tokenizer
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn clone_loses_tokenizer(meta in arb_symbol_metadata_vec(1, 5)) {
        let lang = Language::builder()
            .parse_table(leak_table())
            .symbol_metadata(meta)
            .tokenizer(|_| Box::new(std::iter::empty()))
            .build()
            .unwrap();
        prop_assert!(lang.tokenize.is_some());
        let cloned = lang.clone();
        prop_assert!(cloned.tokenize.is_none());
    }
}

// ---------------------------------------------------------------------------
// 12 – with_static_tokens
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn with_static_tokens_sets_tokenizer_from_vec(
        count in 0usize..20,
    ) {
        let tokens: Vec<Token> = (0..count)
            .map(|i| Token {
                kind: i as u32,
                start: i as u32,
                end: (i + 1) as u32,
            })
            .collect();
        let lang = Language::builder()
            .parse_table(leak_table())
            .symbol_metadata(vec![SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            }])
            .build()
            .unwrap()
            .with_static_tokens(tokens.clone());
        let tok = lang.tokenize.as_ref().unwrap();
        let result: Vec<_> = tok(b"ignored").collect();
        prop_assert_eq!(result.len(), count);
        for i in 0..count {
            prop_assert_eq!(result[i].kind, tokens[i].kind);
        }
    }
}
