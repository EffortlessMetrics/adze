#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for Language builder patterns in adze-runtime.

use adze_runtime::Token;
use adze_runtime::language::{Language, SymbolMetadata};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn leak_table() -> &'static adze_glr_core::ParseTable {
    Box::leak(Box::new(adze_glr_core::ParseTable::default()))
}

fn terminal_meta() -> SymbolMetadata {
    SymbolMetadata {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    }
}

fn nonterminal_meta() -> SymbolMetadata {
    SymbolMetadata {
        is_terminal: false,
        is_visible: true,
        is_supertype: false,
    }
}

fn hidden_meta() -> SymbolMetadata {
    SymbolMetadata {
        is_terminal: true,
        is_visible: false,
        is_supertype: false,
    }
}

fn supertype_meta() -> SymbolMetadata {
    SymbolMetadata {
        is_terminal: false,
        is_visible: true,
        is_supertype: true,
    }
}

// ---------------------------------------------------------------------------
// 1. Builder chain patterns
// ---------------------------------------------------------------------------

#[test]
fn build_with_only_required_fields() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .build();
    assert!(lang.is_ok());
}

#[test]
fn build_full_chain() {
    let lang = Language::builder()
        .version(15)
        .parse_table(leak_table())
        .symbol_names(vec!["eof".into(), "expr".into()])
        .symbol_metadata(vec![hidden_meta(), nonterminal_meta()])
        .field_names(vec!["left".into(), "right".into()])
        .max_alias_sequence_length(3)
        .tokenizer(|_| Box::new(std::iter::empty()))
        .build();
    assert!(lang.is_ok());
    let lang = lang.unwrap();
    assert_eq!(lang.version, 15);
    assert_eq!(lang.symbol_count, 2);
    assert_eq!(lang.field_count, 2);
    assert_eq!(lang.max_alias_sequence_length, 3);
}

#[test]
fn builder_methods_can_be_called_in_any_order() {
    // metadata before parse_table
    let a = Language::builder()
        .symbol_metadata(vec![terminal_meta()])
        .parse_table(leak_table())
        .build();
    assert!(a.is_ok());

    // version last
    let b = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .version(42)
        .build();
    assert!(b.is_ok());
    assert_eq!(b.unwrap().version, 42);
}

#[test]
fn builder_is_consumed_on_build() {
    // Ensure builder follows move semantics — this is a compile-time check;
    // if it compiles, it works.
    let builder = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()]);
    let _lang = builder.build().unwrap();
}

#[test]
fn version_defaults_to_zero() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .build()
        .unwrap();
    assert_eq!(lang.version, 0);
}

#[test]
fn max_alias_defaults_to_zero() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .build()
        .unwrap();
    assert_eq!(lang.max_alias_sequence_length, 0);
}

// ---------------------------------------------------------------------------
// 2. Missing required fields → error
// ---------------------------------------------------------------------------

#[test]
fn missing_parse_table_returns_error() {
    let res = Language::builder()
        .symbol_metadata(vec![terminal_meta()])
        .build();
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), "missing parse table");
}

#[test]
fn missing_symbol_metadata_returns_error() {
    let res = Language::builder().parse_table(leak_table()).build();
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), "missing symbol metadata");
}

#[test]
fn missing_both_required_fields() {
    let res = Language::builder().build();
    // First missing field checked is parse_table
    assert_eq!(res.unwrap_err(), "missing parse table");
}

#[test]
fn missing_parse_table_even_with_optional_fields() {
    let res = Language::builder()
        .version(15)
        .symbol_names(vec!["a".into()])
        .symbol_metadata(vec![terminal_meta()])
        .field_names(vec!["f".into()])
        .build();
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), "missing parse table");
}

#[test]
fn missing_metadata_even_with_optional_fields() {
    let res = Language::builder()
        .version(15)
        .parse_table(leak_table())
        .symbol_names(vec!["a".into()])
        .field_names(vec!["f".into()])
        .build();
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), "missing symbol metadata");
}

// ---------------------------------------------------------------------------
// 3. Different tokenizer implementations
// ---------------------------------------------------------------------------

#[test]
fn tokenizer_returning_empty_iterator() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .tokenizer(|_| Box::new(std::iter::empty()))
        .build()
        .unwrap();
    assert!(lang.tokenize.is_some());
}

#[test]
fn tokenizer_returning_single_token() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .tokenizer(|_| {
            Box::new(std::iter::once(Token {
                kind: 1,
                start: 0,
                end: 3,
            }))
        })
        .build()
        .unwrap();
    let tokenize = lang.tokenize.as_ref().unwrap();
    let tokens: Vec<_> = tokenize(b"abc").collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, 1);
    assert_eq!(tokens[0].start, 0);
    assert_eq!(tokens[0].end, 3);
}

#[test]
fn tokenizer_produces_multiple_tokens() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta(), terminal_meta()])
        .tokenizer(|input| {
            let mut tokens = Vec::new();
            for (i, &b) in input.iter().enumerate() {
                tokens.push(Token {
                    kind: b as u32,
                    start: i as u32,
                    end: (i + 1) as u32,
                });
            }
            Box::new(tokens.into_iter())
        })
        .build()
        .unwrap();
    let tokenize = lang.tokenize.as_ref().unwrap();
    let tokens: Vec<_> = tokenize(b"hi").collect();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, b'h' as u32);
    assert_eq!(tokens[1].kind, b'i' as u32);
}

#[test]
fn tokenizer_uses_input_length() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .tokenizer(|input| {
            // Produce one token spanning the entire input
            Box::new(std::iter::once(Token {
                kind: 0,
                start: 0,
                end: input.len() as u32,
            }))
        })
        .build()
        .unwrap();
    let tokenize = lang.tokenize.as_ref().unwrap();
    let tokens: Vec<_> = tokenize(b"hello world").collect();
    assert_eq!(tokens[0].end, 11);
}

#[test]
fn no_tokenizer_is_valid_for_build() {
    // Builder should succeed without a tokenizer — it's only needed at parse time.
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .build()
        .unwrap();
    assert!(lang.tokenize.is_none());
}

// ---------------------------------------------------------------------------
// 4. Symbol metadata configurations
// ---------------------------------------------------------------------------

#[test]
fn all_terminal_symbols() {
    let meta = vec![terminal_meta(); 5];
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(meta)
        .build()
        .unwrap();
    for i in 0..5 {
        assert!(lang.is_terminal(i as u16));
        assert!(lang.is_visible(i as u16));
    }
}

#[test]
fn all_nonterminal_symbols() {
    let meta = vec![nonterminal_meta(); 3];
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(meta)
        .build()
        .unwrap();
    for i in 0..3 {
        assert!(!lang.is_terminal(i as u16));
        assert!(lang.is_visible(i as u16));
    }
}

#[test]
fn mixed_symbol_metadata() {
    let meta = vec![
        hidden_meta(),
        terminal_meta(),
        nonterminal_meta(),
        supertype_meta(),
    ];
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(meta)
        .build()
        .unwrap();
    // hidden terminal
    assert!(lang.is_terminal(0));
    assert!(!lang.is_visible(0));
    // visible terminal
    assert!(lang.is_terminal(1));
    assert!(lang.is_visible(1));
    // nonterminal
    assert!(!lang.is_terminal(2));
    assert!(lang.is_visible(2));
    // supertype (nonterminal, visible)
    assert!(!lang.is_terminal(3));
    assert!(lang.is_visible(3));
}

#[test]
fn empty_symbol_metadata() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 0);
    // Out-of-range queries should return false
    assert!(!lang.is_terminal(0));
    assert!(!lang.is_visible(0));
}

#[test]
fn supertype_metadata_preserved() {
    let meta = vec![supertype_meta()];
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(meta)
        .build()
        .unwrap();
    assert!(lang.symbol_metadata[0].is_supertype);
    assert!(!lang.symbol_metadata[0].is_terminal);
    assert!(lang.symbol_metadata[0].is_visible);
}

#[test]
fn large_symbol_metadata() {
    let meta: Vec<_> = (0..256)
        .map(|i| SymbolMetadata {
            is_terminal: i % 2 == 0,
            is_visible: i % 3 != 0,
            is_supertype: i % 7 == 0,
        })
        .collect();
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(meta)
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 256);
    assert!(lang.is_terminal(0));
    assert!(!lang.is_terminal(1));
    assert!(lang.symbol_metadata[7].is_supertype);
}

// ---------------------------------------------------------------------------
// 5. Symbol and field name configurations
// ---------------------------------------------------------------------------

#[test]
fn symbol_names_default_to_empty_strings() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta(), nonterminal_meta()])
        .build()
        .unwrap();
    // When no symbol_names are provided, defaults to empty strings
    assert_eq!(lang.symbol_name(0), Some(""));
    assert_eq!(lang.symbol_name(1), Some(""));
}

#[test]
fn field_names_default_to_empty() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .build()
        .unwrap();
    assert_eq!(lang.field_count, 0);
    assert!(lang.field_name(0).is_none());
}

#[test]
fn symbol_name_out_of_range_returns_none() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec!["only_one".into()])
        .symbol_metadata(vec![terminal_meta()])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_name(0), Some("only_one"));
    assert!(lang.symbol_name(1).is_none());
    assert!(lang.symbol_name(999).is_none());
}

#[test]
fn field_name_out_of_range_returns_none() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .field_names(vec!["x".into()])
        .build()
        .unwrap();
    assert_eq!(lang.field_name(0), Some("x"));
    assert!(lang.field_name(1).is_none());
    assert!(lang.field_name(u16::MAX).is_none());
}

#[test]
fn many_field_names() {
    let names: Vec<String> = (0..50).map(|i| format!("field_{i}")).collect();
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .field_names(names.clone())
        .build()
        .unwrap();
    assert_eq!(lang.field_count, 50);
    for i in 0..50 {
        assert_eq!(lang.field_name(i as u16), Some(names[i].as_str()));
    }
}

// ---------------------------------------------------------------------------
// 6. ParseTable variations
// ---------------------------------------------------------------------------

#[test]
fn default_parse_table_has_zero_states() {
    let table = leak_table();
    assert_eq!(table.state_count, 0);
}

#[test]
fn different_parse_tables_can_be_used() {
    // Two separate tables produce two separate languages.
    let t1 = leak_table();
    let t2 = leak_table();

    let l1 = Language::builder()
        .parse_table(t1)
        .symbol_metadata(vec![terminal_meta()])
        .build()
        .unwrap();
    let l2 = Language::builder()
        .parse_table(t2)
        .symbol_metadata(vec![terminal_meta(), nonterminal_meta()])
        .build()
        .unwrap();
    assert_eq!(l1.symbol_count, 1);
    assert_eq!(l2.symbol_count, 2);
}

#[test]
fn parse_table_is_stored_as_some() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .build()
        .unwrap();
    assert!(lang.parse_table.is_some());
}

// ---------------------------------------------------------------------------
// 7. Language reuse after build
// ---------------------------------------------------------------------------

#[test]
fn language_can_be_cloned() {
    let lang = Language::builder()
        .version(10)
        .parse_table(leak_table())
        .symbol_names(vec!["a".into(), "b".into()])
        .symbol_metadata(vec![terminal_meta(), nonterminal_meta()])
        .field_names(vec!["f1".into()])
        .build()
        .unwrap();
    let cloned = lang.clone();
    assert_eq!(cloned.version, 10);
    assert_eq!(cloned.symbol_count, 2);
    assert_eq!(cloned.field_count, 1);
    assert_eq!(cloned.symbol_name(0), Some("a"));
}

#[test]
fn cloned_language_loses_tokenizer() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .tokenizer(|_| Box::new(std::iter::empty()))
        .build()
        .unwrap();
    assert!(lang.tokenize.is_some());
    let cloned = lang.clone();
    // Clone cannot carry the closure.
    assert!(cloned.tokenize.is_none());
}

#[test]
fn language_is_debuggable() {
    let lang = Language::builder()
        .version(15)
        .parse_table(leak_table())
        .symbol_names(vec!["end".into()])
        .symbol_metadata(vec![hidden_meta()])
        .build()
        .unwrap();
    let debug = format!("{lang:?}");
    assert!(debug.contains("Language"));
    assert!(debug.contains("version"));
    assert!(debug.contains("15"));
}

#[test]
fn multiple_languages_from_same_table() {
    let table = leak_table();
    let l1 = Language::builder()
        .parse_table(table)
        .symbol_metadata(vec![terminal_meta()])
        .build()
        .unwrap();
    let l2 = Language::builder()
        .parse_table(table)
        .symbol_metadata(vec![terminal_meta(), nonterminal_meta()])
        .build()
        .unwrap();
    assert_eq!(l1.symbol_count, 1);
    assert_eq!(l2.symbol_count, 2);
}

// ---------------------------------------------------------------------------
// 8. Error messages for invalid configurations
// ---------------------------------------------------------------------------

#[test]
fn error_message_is_static_str() {
    let err: &'static str = Language::builder().build().unwrap_err();
    assert!(!err.is_empty());
}

#[test]
fn error_variants_are_distinct() {
    let e1 = Language::builder()
        .symbol_metadata(vec![terminal_meta()])
        .build()
        .unwrap_err();
    let e2 = Language::builder()
        .parse_table(leak_table())
        .build()
        .unwrap_err();
    assert_ne!(e1, e2);
}

// ---------------------------------------------------------------------------
// 9. with_static_tokens helper
// ---------------------------------------------------------------------------

#[test]
fn with_static_tokens_sets_tokenizer() {
    let tokens = vec![
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
    ];
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .build()
        .unwrap()
        .with_static_tokens(tokens.clone());
    assert!(lang.tokenize.is_some());
    let tokenize = lang.tokenize.as_ref().unwrap();
    let result: Vec<_> = tokenize(b"any input ignored").collect();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].kind, 1);
}

#[test]
fn with_static_tokens_replaces_existing_tokenizer() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .tokenizer(|_| Box::new(std::iter::empty()))
        .build()
        .unwrap()
        .with_static_tokens(vec![Token {
            kind: 42,
            start: 0,
            end: 1,
        }]);
    let tokenize = lang.tokenize.as_ref().unwrap();
    let result: Vec<_> = tokenize(b"x").collect();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].kind, 42);
}

// ---------------------------------------------------------------------------
// 10. Edge cases
// ---------------------------------------------------------------------------

#[test]
fn version_max_u32() {
    let lang = Language::builder()
        .version(u32::MAX)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta()])
        .build()
        .unwrap();
    assert_eq!(lang.version, u32::MAX);
}

#[test]
fn symbol_count_matches_metadata_length() {
    for n in [0, 1, 5, 100] {
        let meta = vec![terminal_meta(); n];
        let lang = Language::builder()
            .parse_table(leak_table())
            .symbol_metadata(meta)
            .build()
            .unwrap();
        assert_eq!(lang.symbol_count, n as u32);
    }
}

#[test]
fn symbol_names_padded_to_metadata_length_when_omitted() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_meta(); 3])
        .build()
        .unwrap();
    // Default names should be empty strings, one per symbol
    assert_eq!(lang.symbol_names.len(), 3);
    for name in &lang.symbol_names {
        assert!(name.is_empty());
    }
}
