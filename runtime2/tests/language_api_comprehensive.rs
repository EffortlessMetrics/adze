#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for the Language API surface in `runtime2/src/language.rs`.
//!
//! Covers builder construction, validation, language queries, symbol metadata,
//! field mapping, tokenizer configuration, parse table integration, edge cases,
//! symbol type queries, and multiple language instances.

use adze_runtime::Token;
use adze_runtime::language::{Language, SymbolMetadata};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn terminal_visible() -> SymbolMetadata {
    SymbolMetadata {
        is_terminal: true,
        is_visible: true,
        is_supertype: false,
    }
}

fn terminal_hidden() -> SymbolMetadata {
    SymbolMetadata {
        is_terminal: true,
        is_visible: false,
        is_supertype: false,
    }
}

fn nonterminal_visible() -> SymbolMetadata {
    SymbolMetadata {
        is_terminal: false,
        is_visible: true,
        is_supertype: false,
    }
}

fn nonterminal_hidden() -> SymbolMetadata {
    SymbolMetadata {
        is_terminal: false,
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

fn leak_table() -> &'static adze_glr_core::ParseTable {
    Box::leak(Box::new(adze_glr_core::ParseTable::default()))
}

fn minimal_language() -> Language {
    Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap()
}

// ===========================================================================
// 1. Builder construction and validation
// ===========================================================================

#[test]
fn builder_succeeds_with_required_fields_only() {
    let result = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build();
    assert!(result.is_ok());
}

#[test]
fn builder_succeeds_with_all_fields() {
    let lang = Language::builder()
        .version(15)
        .max_alias_sequence_length(4)
        .parse_table(leak_table())
        .symbol_names(vec!["end".into(), "number".into(), "expr".into()])
        .symbol_metadata(vec![
            terminal_hidden(),
            terminal_visible(),
            nonterminal_visible(),
        ])
        .field_names(vec!["left".into(), "right".into()])
        .build();
    assert!(lang.is_ok());
}

#[test]
fn builder_method_chaining_order_is_irrelevant() {
    let lang = Language::builder()
        .field_names(vec!["f".into()])
        .symbol_metadata(vec![terminal_visible()])
        .symbol_names(vec!["tok".into()])
        .max_alias_sequence_length(2)
        .version(3)
        .parse_table(leak_table())
        .build()
        .unwrap();

    assert_eq!(lang.version, 3);
    assert_eq!(lang.max_alias_sequence_length, 2);
    assert_eq!(lang.symbol_name(0), Some("tok"));
    assert_eq!(lang.field_name(0), Some("f"));
}

#[test]
fn builder_default_produces_empty_builder() {
    // LanguageBuilder::default() is equivalent to Language::builder()
    let result = Language::builder().build();
    assert!(result.is_err());
}

// ===========================================================================
// 2. Build errors — missing required fields
// ===========================================================================

#[test]
fn build_error_missing_parse_table() {
    let err = Language::builder()
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap_err();
    assert_eq!(err, "missing parse table");
}

#[test]
fn build_error_missing_symbol_metadata() {
    let err = Language::builder()
        .parse_table(leak_table())
        .build()
        .unwrap_err();
    assert_eq!(err, "missing symbol metadata");
}

#[test]
fn build_error_missing_both_required_fields() {
    let result = Language::builder().version(1).build();
    assert!(result.is_err());
}

// ===========================================================================
// 3. Language queries — version
// ===========================================================================

#[test]
fn version_defaults_to_zero() {
    assert_eq!(minimal_language().version, 0);
}

#[test]
fn version_set_via_builder() {
    let lang = Language::builder()
        .version(15)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.version, 15);
}

#[test]
fn version_accepts_large_value() {
    let lang = Language::builder()
        .version(u32::MAX)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.version, u32::MAX);
}

// ===========================================================================
// 4. Language queries — symbol_count
// ===========================================================================

#[test]
fn symbol_count_derived_from_symbol_names_length() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec!["a".into(), "b".into(), "c".into()])
        .symbol_metadata(vec![
            terminal_hidden(),
            terminal_visible(),
            nonterminal_visible(),
        ])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 3);
}

#[test]
fn symbol_count_derived_from_metadata_when_names_omitted() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .build()
        .unwrap();
    // symbol_names defaults to vec of empty strings with same len as metadata
    assert_eq!(lang.symbol_count, 2);
}

// ===========================================================================
// 5. Language queries — symbol_name
// ===========================================================================

#[test]
fn symbol_name_returns_correct_value() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec!["end".into(), "identifier".into(), "number".into()])
        .symbol_metadata(vec![
            terminal_hidden(),
            terminal_visible(),
            terminal_visible(),
        ])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_name(0), Some("end"));
    assert_eq!(lang.symbol_name(1), Some("identifier"));
    assert_eq!(lang.symbol_name(2), Some("number"));
}

#[test]
fn symbol_name_out_of_bounds_returns_none() {
    let lang = minimal_language();
    assert_eq!(lang.symbol_name(999), None);
}

#[test]
fn symbol_name_defaults_to_empty_strings() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_name(0), Some(""));
    assert_eq!(lang.symbol_name(1), Some(""));
}

// ===========================================================================
// 6. Symbol metadata lookup
// ===========================================================================

#[test]
fn symbol_metadata_direct_field_access() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![
            terminal_hidden(),
            nonterminal_visible(),
            supertype_meta(),
        ])
        .build()
        .unwrap();

    assert!(lang.symbol_metadata[0].is_terminal);
    assert!(!lang.symbol_metadata[0].is_visible);
    assert!(!lang.symbol_metadata[0].is_supertype);

    assert!(!lang.symbol_metadata[1].is_terminal);
    assert!(lang.symbol_metadata[1].is_visible);
    assert!(!lang.symbol_metadata[1].is_supertype);

    assert!(!lang.symbol_metadata[2].is_terminal);
    assert!(lang.symbol_metadata[2].is_visible);
    assert!(lang.symbol_metadata[2].is_supertype);
}

#[test]
fn symbol_metadata_via_is_terminal_method() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_visible(), nonterminal_hidden()])
        .build()
        .unwrap();
    assert!(lang.is_terminal(0));
    assert!(!lang.is_terminal(1));
}

#[test]
fn symbol_metadata_via_is_visible_method() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden(), nonterminal_visible()])
        .build()
        .unwrap();
    assert!(!lang.is_visible(0));
    assert!(lang.is_visible(1));
}

// ===========================================================================
// 7. Field name/id mapping
// ===========================================================================

#[test]
fn field_name_returns_correct_value() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec![
            "condition".into(),
            "body".into(),
            "alternative".into(),
        ])
        .build()
        .unwrap();
    assert_eq!(lang.field_name(0), Some("condition"));
    assert_eq!(lang.field_name(1), Some("body"));
    assert_eq!(lang.field_name(2), Some("alternative"));
}

#[test]
fn field_name_out_of_bounds_returns_none() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["x".into()])
        .build()
        .unwrap();
    assert_eq!(lang.field_name(1), None);
    assert_eq!(lang.field_name(100), None);
}

#[test]
fn field_count_matches_field_names_len() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["a".into(), "b".into()])
        .build()
        .unwrap();
    assert_eq!(lang.field_count, 2);
}

#[test]
fn field_count_zero_when_no_fields_provided() {
    let lang = minimal_language();
    assert_eq!(lang.field_count, 0);
    assert_eq!(lang.field_name(0), None);
}

#[test]
fn field_id_lookup_by_iterating_field_names() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["left".into(), "operator".into(), "right".into()])
        .build()
        .unwrap();

    // Simulate field_id_for_name by searching field_names
    let target = "operator";
    let found_id = lang
        .field_names
        .iter()
        .position(|n| n == target)
        .map(|i| i as u16);
    assert_eq!(found_id, Some(1));
}

// ===========================================================================
// 8. Tokenizer configuration
// ===========================================================================

#[cfg(feature = "glr-core")]
#[test]
fn tokenizer_can_be_set_via_builder() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .tokenizer(|_input: &[u8]| Box::new(std::iter::empty::<Token>()))
        .build()
        .unwrap();
    assert!(lang.tokenize.is_some());
}

#[cfg(feature = "glr-core")]
#[test]
fn tokenizer_absent_by_default() {
    let lang = minimal_language();
    assert!(lang.tokenize.is_none());
}

#[cfg(feature = "glr-core")]
#[test]
fn with_static_tokens_sets_tokenizer() {
    let lang = minimal_language().with_static_tokens(vec![
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
    ]);
    assert!(lang.tokenize.is_some());
}

#[cfg(feature = "glr-core")]
#[test]
fn tokenizer_produces_expected_tokens() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .tokenizer(|_input: &[u8]| {
            Box::new(
                vec![
                    Token {
                        kind: 1,
                        start: 0,
                        end: 3,
                    },
                    Token {
                        kind: 0,
                        start: 3,
                        end: 3,
                    },
                ]
                .into_iter(),
            )
        })
        .build()
        .unwrap();

    let tokenize_fn = lang.tokenize.as_ref().unwrap();
    let tokens: Vec<Token> = tokenize_fn(b"abc").collect();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, 1);
    assert_eq!(tokens[0].start, 0);
    assert_eq!(tokens[0].end, 3);
    assert_eq!(tokens[1].kind, 0);
}

// ===========================================================================
// 9. ParseTable integration
// ===========================================================================

#[test]
fn parse_table_default_can_be_leaked_and_used() {
    let table: &'static adze_glr_core::ParseTable =
        Box::leak(Box::new(adze_glr_core::ParseTable::default()));
    assert_eq!(table.state_count, 0);
    assert_eq!(table.symbol_count, 0);

    let lang = Language::builder()
        .parse_table(table)
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert!(lang.parse_table.is_some());
}

#[test]
fn parse_table_with_custom_state_count() {
    let table = adze_glr_core::ParseTable {
        state_count: 42,
        ..adze_glr_core::ParseTable::default()
    };
    let table_ref: &'static adze_glr_core::ParseTable = Box::leak(Box::new(table));

    let lang = Language::builder()
        .parse_table(table_ref)
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();

    assert_eq!(lang.parse_table.unwrap().state_count, 42);
}

// ===========================================================================
// 10. Edge cases — zero symbols, large counts, boundary indices
// ===========================================================================

#[test]
fn zero_symbols_language() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 0);
    assert_eq!(lang.symbol_name(0), None);
    assert!(!lang.is_terminal(0));
    assert!(!lang.is_visible(0));
}

#[test]
fn large_symbol_count() {
    let n = 1000usize;
    let names: Vec<String> = (0..n).map(|i| format!("sym_{i}")).collect();
    let meta: Vec<SymbolMetadata> = (0..n).map(|_| terminal_visible()).collect();
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(names)
        .symbol_metadata(meta)
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, n as u32);
    assert_eq!(lang.symbol_name(0), Some("sym_0"));
    assert_eq!(lang.symbol_name(999), Some("sym_999"));
    assert_eq!(lang.symbol_name(1000), None);
}

#[test]
fn symbol_name_at_exact_boundary() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec!["only".into()])
        .symbol_metadata(vec![terminal_visible()])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_name(0), Some("only"));
    assert_eq!(lang.symbol_name(1), None);
}

#[test]
fn is_terminal_out_of_bounds_returns_false() {
    let lang = minimal_language();
    assert!(!lang.is_terminal(u16::MAX));
}

#[test]
fn is_visible_out_of_bounds_returns_false() {
    let lang = minimal_language();
    assert!(!lang.is_visible(u16::MAX));
}

// ===========================================================================
// 11. Symbol type queries — is_terminal, is_visible, is_supertype
// ===========================================================================

#[test]
fn mixed_symbol_types_comprehensive() {
    let meta = vec![
        terminal_hidden(),     // 0: EOF-like hidden terminal
        terminal_visible(),    // 1: visible token (e.g., keyword)
        nonterminal_visible(), // 2: named rule (e.g., expression)
        nonterminal_hidden(),  // 3: hidden rule (e.g., _expression)
        supertype_meta(),      // 4: supertype (e.g., _statement)
    ];
    let names = vec![
        "end".into(),
        "number".into(),
        "expression".into(),
        "_rule".into(),
        "_statement".into(),
    ];
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(names)
        .symbol_metadata(meta)
        .build()
        .unwrap();

    // terminal + hidden
    assert!(lang.is_terminal(0));
    assert!(!lang.is_visible(0));
    assert!(!lang.symbol_metadata[0].is_supertype);

    // terminal + visible
    assert!(lang.is_terminal(1));
    assert!(lang.is_visible(1));

    // nonterminal + visible
    assert!(!lang.is_terminal(2));
    assert!(lang.is_visible(2));

    // nonterminal + hidden
    assert!(!lang.is_terminal(3));
    assert!(!lang.is_visible(3));

    // supertype
    assert!(!lang.is_terminal(4));
    assert!(lang.is_visible(4));
    assert!(lang.symbol_metadata[4].is_supertype);
}

#[test]
fn all_four_combinations_of_terminal_and_visible() {
    let meta = vec![
        SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: true,
            is_visible: false,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: false,
            is_visible: true,
            is_supertype: false,
        },
        SymbolMetadata {
            is_terminal: false,
            is_visible: false,
            is_supertype: false,
        },
    ];
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(meta)
        .build()
        .unwrap();

    assert!(lang.is_terminal(0) && lang.is_visible(0));
    assert!(lang.is_terminal(1) && !lang.is_visible(1));
    assert!(!lang.is_terminal(2) && lang.is_visible(2));
    assert!(!lang.is_terminal(3) && !lang.is_visible(3));
}

#[test]
fn supertype_is_stored_and_retrievable() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: true,
            },
            SymbolMetadata {
                is_terminal: false,
                is_visible: true,
                is_supertype: false,
            },
        ])
        .build()
        .unwrap();
    assert!(lang.symbol_metadata[0].is_supertype);
    assert!(!lang.symbol_metadata[1].is_supertype);
}

// ===========================================================================
// 12. Multiple language instances
// ===========================================================================

#[test]
fn two_languages_are_independent() {
    let lang_a = Language::builder()
        .version(1)
        .parse_table(leak_table())
        .symbol_names(vec!["a_sym".into()])
        .symbol_metadata(vec![terminal_visible()])
        .field_names(vec!["a_field".into()])
        .build()
        .unwrap();

    let lang_b = Language::builder()
        .version(2)
        .parse_table(leak_table())
        .symbol_names(vec!["b_sym".into(), "b_sym2".into()])
        .symbol_metadata(vec![nonterminal_visible(), terminal_hidden()])
        .build()
        .unwrap();

    assert_eq!(lang_a.version, 1);
    assert_eq!(lang_b.version, 2);
    assert_eq!(lang_a.symbol_count, 1);
    assert_eq!(lang_b.symbol_count, 2);
    assert_eq!(lang_a.field_count, 1);
    assert_eq!(lang_b.field_count, 0);
    assert_eq!(lang_a.symbol_name(0), Some("a_sym"));
    assert_eq!(lang_b.symbol_name(0), Some("b_sym"));
}

#[test]
fn cloned_language_is_independent() {
    let original = Language::builder()
        .version(10)
        .parse_table(leak_table())
        .symbol_names(vec!["alpha".into(), "beta".into()])
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .field_names(vec!["field_a".into()])
        .build()
        .unwrap();

    let cloned = original.clone();
    assert_eq!(cloned.version, 10);
    assert_eq!(cloned.symbol_count, 2);
    assert_eq!(cloned.field_count, 1);
    assert_eq!(cloned.symbol_name(0), Some("alpha"));
    assert_eq!(cloned.symbol_name(1), Some("beta"));
    assert_eq!(cloned.field_name(0), Some("field_a"));
}

// ===========================================================================
// 13. max_alias_sequence_length
// ===========================================================================

#[test]
fn max_alias_sequence_length_defaults_to_zero() {
    assert_eq!(minimal_language().max_alias_sequence_length, 0);
}

#[test]
fn max_alias_sequence_length_set_via_builder() {
    let lang = Language::builder()
        .max_alias_sequence_length(7)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.max_alias_sequence_length, 7);
}

// ===========================================================================
// 14. Debug formatting
// ===========================================================================

#[test]
fn debug_format_is_nonempty_and_starts_with_language() {
    let lang = minimal_language();
    let debug = format!("{lang:?}");
    assert!(!debug.is_empty());
    assert!(debug.starts_with("Language"), "got: {debug}");
}

#[test]
fn debug_format_includes_version_and_counts() {
    let lang = Language::builder()
        .version(15)
        .parse_table(leak_table())
        .symbol_names(vec!["tok".into()])
        .symbol_metadata(vec![terminal_visible()])
        .field_names(vec!["f1".into(), "f2".into()])
        .build()
        .unwrap();
    let debug = format!("{lang:?}");
    assert!(debug.contains("version: 15"), "debug: {debug}");
    assert!(debug.contains("symbol_count: 1"), "debug: {debug}");
    assert!(debug.contains("field_count: 2"), "debug: {debug}");
}

// ===========================================================================
// 15. Token is Copy and fields are accessible
// ===========================================================================

#[test]
fn token_is_copy() {
    let t = Token {
        kind: 5,
        start: 10,
        end: 20,
    };
    let t2 = t; // Copy
    assert_eq!(t.kind, t2.kind);
    assert_eq!(t.start, t2.start);
    assert_eq!(t.end, t2.end);
}

#[test]
fn token_debug_format() {
    let t = Token {
        kind: 1,
        start: 0,
        end: 3,
    };
    let debug = format!("{t:?}");
    assert!(debug.contains("kind: 1"), "debug: {debug}");
}

// ===========================================================================
// 16. Unicode and special characters in names
// ===========================================================================

#[test]
fn symbol_names_with_unicode() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec!["λ".into(), "→".into(), "∀".into()])
        .symbol_metadata(vec![
            terminal_visible(),
            terminal_visible(),
            terminal_visible(),
        ])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_name(0), Some("λ"));
    assert_eq!(lang.symbol_name(1), Some("→"));
    assert_eq!(lang.symbol_name(2), Some("∀"));
}

#[test]
fn field_names_with_special_characters() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["my-field".into(), "my_field_2".into(), "".into()])
        .build()
        .unwrap();
    assert_eq!(lang.field_name(0), Some("my-field"));
    assert_eq!(lang.field_name(1), Some("my_field_2"));
    assert_eq!(lang.field_name(2), Some(""));
    assert_eq!(lang.field_count, 3);
}

// ===========================================================================
// 17. symbol_for_name pattern (iterate symbol_names)
// ===========================================================================

#[test]
fn find_symbol_id_by_name() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec![
            "end".into(),
            "number".into(),
            "plus".into(),
            "expr".into(),
        ])
        .symbol_metadata(vec![
            terminal_hidden(),
            terminal_visible(),
            terminal_visible(),
            nonterminal_visible(),
        ])
        .build()
        .unwrap();

    // Search for "plus" symbol id
    let id = lang.symbol_names.iter().position(|n| n == "plus");
    assert_eq!(id, Some(2));

    // Confirm it's a terminal
    assert!(lang.is_terminal(id.unwrap() as u16));

    // Search for nonexistent symbol
    let missing = lang.symbol_names.iter().position(|n| n == "missing");
    assert_eq!(missing, None);
}

// ===========================================================================
// 18. Clone drops tokenizer
// ===========================================================================

#[cfg(feature = "glr-core")]
#[test]
fn clone_resets_tokenizer_to_none() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .tokenizer(|_: &[u8]| Box::new(std::iter::empty::<Token>()))
        .build()
        .unwrap();
    assert!(lang.tokenize.is_some());

    let cloned = lang.clone();
    // Closures can't be cloned, so tokenize is reset to None
    assert!(cloned.tokenize.is_none());
}

// ===========================================================================
// 19. Consistency between symbol_count and metadata length
// ===========================================================================

#[test]
fn symbol_count_equals_metadata_length() {
    for n in [0, 1, 5, 50] {
        let meta: Vec<SymbolMetadata> = (0..n).map(|_| terminal_visible()).collect();
        let expected = n as u32;
        let lang = Language::builder()
            .parse_table(leak_table())
            .symbol_metadata(meta)
            .build()
            .unwrap();
        assert_eq!(lang.symbol_count, expected, "n = {n}");
        assert_eq!(lang.symbol_metadata.len(), n);
    }
}
