//! Comprehensive tests (v2) for the Language API surface in `runtime2/src/language.rs`.
//!
//! Covers: LanguageBuilder construction & validation, Language properties,
//! symbol name/metadata lookups, field name lookups, symbol_for_name,
//! clone behaviour, Debug formatting, edge cases, and cross-instance isolation.

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

/// Build a language with the given symbol names and metadata.
fn language_with_symbols(names: Vec<&str>, meta: Vec<SymbolMetadata>) -> Language {
    Language::builder()
        .parse_table(leak_table())
        .symbol_names(names.into_iter().map(String::from).collect())
        .symbol_metadata(meta)
        .build()
        .unwrap()
}

// ===========================================================================
// 1. Builder – required fields
// ===========================================================================

#[test]
fn build_ok_with_parse_table_and_metadata() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_visible()])
        .build();
    assert!(lang.is_ok());
}

#[test]
fn build_err_when_parse_table_missing() {
    let err = Language::builder()
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap_err();
    assert_eq!(err, "missing parse table");
}

#[test]
fn build_err_when_metadata_missing() {
    let err = Language::builder()
        .parse_table(leak_table())
        .build()
        .unwrap_err();
    assert_eq!(err, "missing symbol metadata");
}

#[test]
fn build_err_when_both_required_missing() {
    assert!(Language::builder().build().is_err());
}

// ===========================================================================
// 2. Builder – optional fields and defaults
// ===========================================================================

#[test]
fn version_defaults_to_zero() {
    assert_eq!(minimal_language().version, 0);
}

#[test]
fn version_set_via_builder() {
    let lang = Language::builder()
        .version(42)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.version, 42);
}

#[test]
fn max_alias_sequence_length_defaults_to_zero() {
    assert_eq!(minimal_language().max_alias_sequence_length, 0);
}

#[test]
fn max_alias_sequence_length_set_via_builder() {
    let lang = Language::builder()
        .max_alias_sequence_length(9)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.max_alias_sequence_length, 9);
}

#[test]
fn symbol_names_default_to_empty_strings_matching_metadata_len() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_names.len(), 2);
    assert_eq!(lang.symbol_name(0), Some(""));
    assert_eq!(lang.symbol_name(1), Some(""));
}

#[test]
fn field_names_default_to_empty_vec() {
    let lang = minimal_language();
    assert!(lang.field_names.is_empty());
    assert_eq!(lang.field_count, 0);
}

// ===========================================================================
// 3. Builder – chaining order independence
// ===========================================================================

#[test]
fn builder_chaining_order_does_not_matter() {
    let lang = Language::builder()
        .field_names(vec!["f".into()])
        .symbol_metadata(vec![terminal_visible()])
        .symbol_names(vec!["tok".into()])
        .max_alias_sequence_length(5)
        .version(7)
        .parse_table(leak_table())
        .build()
        .unwrap();

    assert_eq!(lang.version, 7);
    assert_eq!(lang.max_alias_sequence_length, 5);
    assert_eq!(lang.symbol_name(0), Some("tok"));
    assert_eq!(lang.field_name(0), Some("f"));
}

// ===========================================================================
// 4. symbol_count derivation
// ===========================================================================

#[test]
fn symbol_count_from_explicit_names() {
    let lang = language_with_symbols(
        vec!["a", "b", "c"],
        vec![terminal_visible(), terminal_hidden(), nonterminal_visible()],
    );
    assert_eq!(lang.symbol_count, 3);
}

#[test]
fn symbol_count_from_metadata_when_names_omitted() {
    let lang = Language::builder()
        .parse_table(leak_table())
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
fn symbol_count_zero_for_empty_metadata() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 0);
}

// ===========================================================================
// 5. field_count derivation
// ===========================================================================

#[test]
fn field_count_matches_field_names_len() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["a".into(), "b".into(), "c".into()])
        .build()
        .unwrap();
    assert_eq!(lang.field_count, 3);
}

#[test]
fn field_count_zero_when_no_field_names() {
    assert_eq!(minimal_language().field_count, 0);
}

// ===========================================================================
// 6. symbol_name lookup
// ===========================================================================

#[test]
fn symbol_name_returns_expected_values() {
    let lang = language_with_symbols(
        vec!["end", "num", "expr"],
        vec![terminal_hidden(), terminal_visible(), nonterminal_visible()],
    );
    assert_eq!(lang.symbol_name(0), Some("end"));
    assert_eq!(lang.symbol_name(1), Some("num"));
    assert_eq!(lang.symbol_name(2), Some("expr"));
}

#[test]
fn symbol_name_out_of_bounds_returns_none() {
    let lang = minimal_language();
    assert_eq!(lang.symbol_name(1), None);
    assert_eq!(lang.symbol_name(100), None);
    assert_eq!(lang.symbol_name(u16::MAX), None);
}

#[test]
fn symbol_name_with_empty_string() {
    let lang = language_with_symbols(vec!["", "x"], vec![terminal_hidden(), terminal_visible()]);
    assert_eq!(lang.symbol_name(0), Some(""));
}

// ===========================================================================
// 7. field_name lookup
// ===========================================================================

#[test]
fn field_name_returns_expected_values() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["cond".into(), "body".into(), "alt".into()])
        .build()
        .unwrap();
    assert_eq!(lang.field_name(0), Some("cond"));
    assert_eq!(lang.field_name(1), Some("body"));
    assert_eq!(lang.field_name(2), Some("alt"));
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
    assert_eq!(lang.field_name(u16::MAX), None);
}

#[test]
fn field_name_empty_fields_gives_none() {
    assert_eq!(minimal_language().field_name(0), None);
}

// ===========================================================================
// 8. is_terminal
// ===========================================================================

#[test]
fn is_terminal_true_for_terminal_symbol() {
    let lang = language_with_symbols(vec!["t"], vec![terminal_visible()]);
    assert!(lang.is_terminal(0));
}

#[test]
fn is_terminal_false_for_nonterminal_symbol() {
    let lang = language_with_symbols(vec!["nt"], vec![nonterminal_visible()]);
    assert!(!lang.is_terminal(0));
}

#[test]
fn is_terminal_false_out_of_bounds() {
    assert!(!minimal_language().is_terminal(999));
    assert!(!minimal_language().is_terminal(u16::MAX));
}

// ===========================================================================
// 9. is_visible
// ===========================================================================

#[test]
fn is_visible_true_for_visible_symbol() {
    let lang = language_with_symbols(vec!["v"], vec![terminal_visible()]);
    assert!(lang.is_visible(0));
}

#[test]
fn is_visible_false_for_hidden_symbol() {
    let lang = language_with_symbols(vec!["h"], vec![terminal_hidden()]);
    assert!(!lang.is_visible(0));
}

#[test]
fn is_visible_false_out_of_bounds() {
    assert!(!minimal_language().is_visible(999));
    assert!(!minimal_language().is_visible(u16::MAX));
}

// ===========================================================================
// 10. symbol_for_name
// ===========================================================================

#[test]
fn symbol_for_name_finds_named_symbol() {
    let lang = language_with_symbols(
        vec!["end", "number", "expr"],
        vec![terminal_hidden(), terminal_visible(), nonterminal_visible()],
    );
    // "number" is visible (is_named = true)
    assert_eq!(lang.symbol_for_name("number", true), Some(1));
}

#[test]
fn symbol_for_name_finds_anonymous_symbol() {
    let lang = language_with_symbols(
        vec!["end", "number"],
        vec![terminal_hidden(), terminal_visible()],
    );
    // "end" is hidden (is_named = false)
    assert_eq!(lang.symbol_for_name("end", false), Some(0));
}

#[test]
fn symbol_for_name_returns_none_when_not_found() {
    let lang = language_with_symbols(vec!["a"], vec![terminal_visible()]);
    assert_eq!(lang.symbol_for_name("nonexistent", true), None);
}

#[test]
fn symbol_for_name_returns_none_when_visibility_mismatches() {
    let lang = language_with_symbols(vec!["tok"], vec![terminal_hidden()]);
    // "tok" is hidden, but we search for named (visible)
    assert_eq!(lang.symbol_for_name("tok", true), None);
    // And vice-versa:
    let lang2 = language_with_symbols(vec!["tok"], vec![terminal_visible()]);
    assert_eq!(lang2.symbol_for_name("tok", false), None);
}

#[test]
fn symbol_for_name_returns_first_match_on_duplicate_names() {
    let lang = language_with_symbols(
        vec!["dup", "dup"],
        vec![terminal_visible(), terminal_visible()],
    );
    assert_eq!(lang.symbol_for_name("dup", true), Some(0));
}

#[test]
fn symbol_for_name_empty_language() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_for_name("anything", true), None);
    assert_eq!(lang.symbol_for_name("anything", false), None);
}

#[test]
fn symbol_for_name_with_empty_name_string() {
    let lang = language_with_symbols(vec![""], vec![terminal_visible()]);
    assert_eq!(lang.symbol_for_name("", true), Some(0));
}

#[test]
fn symbol_for_name_selects_correct_among_mixed_visibility() {
    let lang = language_with_symbols(vec!["x", "x"], vec![terminal_hidden(), terminal_visible()]);
    assert_eq!(lang.symbol_for_name("x", false), Some(0)); // hidden one
    assert_eq!(lang.symbol_for_name("x", true), Some(1)); // visible one
}

// ===========================================================================
// 11. SymbolMetadata direct field access
// ===========================================================================

#[test]
fn symbol_metadata_all_fields_accessible() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![
            terminal_hidden(),
            nonterminal_visible(),
            supertype_meta(),
        ])
        .build()
        .unwrap();

    let m0 = &lang.symbol_metadata[0];
    assert!(m0.is_terminal);
    assert!(!m0.is_visible);
    assert!(!m0.is_supertype);

    let m1 = &lang.symbol_metadata[1];
    assert!(!m1.is_terminal);
    assert!(m1.is_visible);
    assert!(!m1.is_supertype);

    let m2 = &lang.symbol_metadata[2];
    assert!(!m2.is_terminal);
    assert!(m2.is_visible);
    assert!(m2.is_supertype);
}

#[test]
fn symbol_metadata_copy_semantics() {
    let m = terminal_visible();
    let m2 = m; // Copy
    assert_eq!(m.is_terminal, m2.is_terminal);
    assert_eq!(m.is_visible, m2.is_visible);
    assert_eq!(m.is_supertype, m2.is_supertype);
}

#[test]
fn symbol_metadata_clone() {
    let m = supertype_meta();
    #[allow(clippy::clone_on_copy)]
    let m2 = m.clone();
    assert!(m2.is_supertype);
}

// ===========================================================================
// 12. Clone behaviour
// ===========================================================================

#[test]
fn cloned_language_has_same_version() {
    let original = Language::builder()
        .version(99)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(original.clone().version, 99);
}

#[test]
fn cloned_language_has_same_symbol_names() {
    let original = language_with_symbols(
        vec!["alpha", "beta"],
        vec![terminal_hidden(), terminal_visible()],
    );
    let cloned = original.clone();
    assert_eq!(cloned.symbol_name(0), Some("alpha"));
    assert_eq!(cloned.symbol_name(1), Some("beta"));
}

#[test]
fn cloned_language_has_same_field_names() {
    let original = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["f1".into(), "f2".into()])
        .build()
        .unwrap();
    let cloned = original.clone();
    assert_eq!(cloned.field_name(0), Some("f1"));
    assert_eq!(cloned.field_name(1), Some("f2"));
    assert_eq!(cloned.field_count, 2);
}

#[test]
fn cloned_language_has_same_counts() {
    let original = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec!["a".into(), "b".into()])
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .field_names(vec!["f".into()])
        .build()
        .unwrap();
    let cloned = original.clone();
    assert_eq!(cloned.symbol_count, 2);
    assert_eq!(cloned.field_count, 1);
    assert_eq!(cloned.max_alias_sequence_length, 0);
}

#[cfg(feature = "glr-core")]
#[test]
fn clone_resets_tokenizer_to_none() {
    use adze_runtime::Token;

    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .tokenizer(|_: &[u8]| Box::new(std::iter::empty::<Token>()))
        .build()
        .unwrap();
    assert!(lang.tokenize.is_some());
    assert!(lang.clone().tokenize.is_none());
}

// ===========================================================================
// 13. Debug formatting
// ===========================================================================

#[test]
fn debug_starts_with_language() {
    let debug = format!("{:?}", minimal_language());
    assert!(debug.starts_with("Language"));
}

#[test]
fn debug_contains_version() {
    let lang = Language::builder()
        .version(15)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    let debug = format!("{lang:?}");
    assert!(debug.contains("version: 15"), "got: {debug}");
}

#[test]
fn debug_contains_symbol_and_field_counts() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec!["a".into(), "b".into()])
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .field_names(vec!["f".into()])
        .build()
        .unwrap();
    let debug = format!("{lang:?}");
    assert!(debug.contains("symbol_count: 2"), "got: {debug}");
    assert!(debug.contains("field_count: 1"), "got: {debug}");
}

#[test]
fn debug_contains_symbol_names_vec() {
    let lang = language_with_symbols(vec!["foo"], vec![terminal_visible()]);
    let debug = format!("{lang:?}");
    assert!(debug.contains("foo"), "got: {debug}");
}

// ===========================================================================
// 14. Edge cases — empty language
// ===========================================================================

#[test]
fn empty_language_symbol_name_returns_none() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_name(0), None);
}

#[test]
fn empty_language_is_terminal_returns_false() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert!(!lang.is_terminal(0));
}

#[test]
fn empty_language_is_visible_returns_false() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert!(!lang.is_visible(0));
}

// ===========================================================================
// 15. Edge cases — boundary indices
// ===========================================================================

#[test]
fn symbol_name_at_exact_last_index() {
    let lang = language_with_symbols(vec!["only"], vec![terminal_visible()]);
    assert_eq!(lang.symbol_name(0), Some("only"));
    assert_eq!(lang.symbol_name(1), None);
}

#[test]
fn field_name_at_exact_last_index() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["only".into()])
        .build()
        .unwrap();
    assert_eq!(lang.field_name(0), Some("only"));
    assert_eq!(lang.field_name(1), None);
}

#[test]
fn is_terminal_at_u16_max_returns_false() {
    assert!(!minimal_language().is_terminal(u16::MAX));
}

#[test]
fn is_visible_at_u16_max_returns_false() {
    assert!(!minimal_language().is_visible(u16::MAX));
}

#[test]
fn symbol_for_name_at_u16_max_does_not_panic() {
    // Ensure no overflow even with large symbol arrays
    let lang = minimal_language();
    assert_eq!(lang.symbol_for_name("anything", true), None);
}

// ===========================================================================
// 16. Edge cases — large language
// ===========================================================================

#[test]
fn large_symbol_count_language() {
    let n = 500usize;
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
    assert_eq!(lang.symbol_name(499), Some("sym_499"));
    assert_eq!(lang.symbol_name(500), None);
}

#[test]
fn large_field_count_language() {
    let n = 200usize;
    let fields: Vec<String> = (0..n).map(|i| format!("field_{i}")).collect();
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(fields)
        .build()
        .unwrap();
    assert_eq!(lang.field_count, n as u32);
    assert_eq!(lang.field_name(0), Some("field_0"));
    assert_eq!(lang.field_name(199), Some("field_199"));
    assert_eq!(lang.field_name(200), None);
}

// ===========================================================================
// 17. Unicode and special characters
// ===========================================================================

#[test]
fn symbol_names_with_unicode_characters() {
    let lang = language_with_symbols(
        vec!["λ", "→", "αβγ"],
        vec![terminal_visible(), terminal_visible(), terminal_visible()],
    );
    assert_eq!(lang.symbol_name(0), Some("λ"));
    assert_eq!(lang.symbol_name(1), Some("→"));
    assert_eq!(lang.symbol_name(2), Some("αβγ"));
}

#[test]
fn field_names_with_unicode_characters() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["名前".into(), "값".into()])
        .build()
        .unwrap();
    assert_eq!(lang.field_name(0), Some("名前"));
    assert_eq!(lang.field_name(1), Some("값"));
}

#[test]
fn symbol_for_name_with_unicode() {
    let lang = language_with_symbols(vec!["λ"], vec![terminal_visible()]);
    assert_eq!(lang.symbol_for_name("λ", true), Some(0));
    assert_eq!(lang.symbol_for_name("λ", false), None);
}

// ===========================================================================
// 18. Mixed symbol types
// ===========================================================================

#[test]
fn all_four_terminal_visible_combinations() {
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
fn supertype_flag_preserved() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![supertype_meta(), terminal_visible()])
        .build()
        .unwrap();
    assert!(lang.symbol_metadata[0].is_supertype);
    assert!(!lang.symbol_metadata[1].is_supertype);
}

// ===========================================================================
// 19. Two independent language instances
// ===========================================================================

#[test]
fn two_languages_do_not_interfere() {
    let lang_a = Language::builder()
        .version(1)
        .parse_table(leak_table())
        .symbol_names(vec!["a".into()])
        .symbol_metadata(vec![terminal_visible()])
        .field_names(vec!["fa".into()])
        .build()
        .unwrap();

    let lang_b = Language::builder()
        .version(2)
        .parse_table(leak_table())
        .symbol_names(vec!["b1".into(), "b2".into()])
        .symbol_metadata(vec![nonterminal_visible(), terminal_hidden()])
        .build()
        .unwrap();

    assert_eq!(lang_a.version, 1);
    assert_eq!(lang_b.version, 2);
    assert_eq!(lang_a.symbol_count, 1);
    assert_eq!(lang_b.symbol_count, 2);
    assert_eq!(lang_a.field_count, 1);
    assert_eq!(lang_b.field_count, 0);
    assert_eq!(lang_a.symbol_name(0), Some("a"));
    assert_eq!(lang_b.symbol_name(0), Some("b1"));
}

// ===========================================================================
// 20. version extreme values
// ===========================================================================

#[test]
fn version_u32_max() {
    let lang = Language::builder()
        .version(u32::MAX)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.version, u32::MAX);
}

#[test]
fn version_one() {
    let lang = Language::builder()
        .version(1)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.version, 1);
}

// ===========================================================================
// 21. max_alias_sequence_length extreme values
// ===========================================================================

#[test]
fn max_alias_sequence_length_u32_max() {
    let lang = Language::builder()
        .max_alias_sequence_length(u32::MAX)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.max_alias_sequence_length, u32::MAX);
}

// ===========================================================================
// 22. Tokenizer (glr-core feature)
// ===========================================================================

#[cfg(feature = "glr-core")]
#[test]
fn tokenizer_absent_by_default() {
    assert!(minimal_language().tokenize.is_none());
}

#[cfg(feature = "glr-core")]
#[test]
fn tokenizer_set_via_builder() {
    use adze_runtime::Token;
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .tokenizer(|_: &[u8]| Box::new(std::iter::empty::<Token>()))
        .build()
        .unwrap();
    assert!(lang.tokenize.is_some());
}

#[cfg(feature = "glr-core")]
#[test]
fn tokenizer_produces_tokens() {
    use adze_runtime::Token;
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .tokenizer(|_: &[u8]| {
            Box::new(
                vec![Token {
                    kind: 1,
                    start: 0,
                    end: 5,
                }]
                .into_iter(),
            )
        })
        .build()
        .unwrap();
    let toks: Vec<Token> = (lang.tokenize.as_ref().unwrap())(b"hello").collect();
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].kind, 1);
    assert_eq!(toks[0].end, 5);
}

#[cfg(feature = "glr-core")]
#[test]
fn with_static_tokens_sets_tokenizer() {
    use adze_runtime::Token;
    let lang = minimal_language().with_static_tokens(vec![Token {
        kind: 0,
        start: 0,
        end: 0,
    }]);
    assert!(lang.tokenize.is_some());
}

// ===========================================================================
// 23. ParseTable integration
// ===========================================================================

#[cfg(feature = "glr-core")]
#[test]
fn parse_table_is_stored() {
    let lang = minimal_language();
    assert!(lang.parse_table.is_some());
}

#[cfg(feature = "glr-core")]
#[test]
fn parse_table_state_count_accessible() {
    let table = adze_glr_core::ParseTable {
        state_count: 17,
        ..adze_glr_core::ParseTable::default()
    };
    let table_ref: &'static _ = Box::leak(Box::new(table));
    let lang = Language::builder()
        .parse_table(table_ref)
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.parse_table.unwrap().state_count, 17);
}

// ===========================================================================
// 24. symbol_for_name with nonterminals
// ===========================================================================

#[test]
fn symbol_for_name_finds_nonterminal_named() {
    let lang = language_with_symbols(
        vec!["expression", "statement"],
        vec![nonterminal_visible(), nonterminal_visible()],
    );
    assert_eq!(lang.symbol_for_name("expression", true), Some(0));
    assert_eq!(lang.symbol_for_name("statement", true), Some(1));
}

#[test]
fn symbol_for_name_finds_nonterminal_anonymous() {
    let lang = language_with_symbols(vec!["_hidden_rule"], vec![nonterminal_hidden()]);
    assert_eq!(lang.symbol_for_name("_hidden_rule", false), Some(0));
    assert_eq!(lang.symbol_for_name("_hidden_rule", true), None);
}

// ===========================================================================
// 25. symbol_for_name with supertypes
// ===========================================================================

#[test]
fn symbol_for_name_supertype_is_visible() {
    let lang = language_with_symbols(vec!["_stmt"], vec![supertype_meta()]);
    // Supertypes are visible, so is_named = true should find it
    assert_eq!(lang.symbol_for_name("_stmt", true), Some(0));
    assert_eq!(lang.symbol_for_name("_stmt", false), None);
}

// ===========================================================================
// 26. Consistency checks
// ===========================================================================

#[test]
fn symbol_count_equals_symbol_names_len() {
    let lang = language_with_symbols(
        vec!["a", "b", "c", "d"],
        vec![
            terminal_visible(),
            terminal_hidden(),
            nonterminal_visible(),
            nonterminal_hidden(),
        ],
    );
    assert_eq!(lang.symbol_count as usize, lang.symbol_names.len());
}

#[test]
fn symbol_count_equals_symbol_metadata_len() {
    let lang = language_with_symbols(vec!["a", "b"], vec![terminal_visible(), terminal_hidden()]);
    assert_eq!(lang.symbol_count as usize, lang.symbol_metadata.len());
}

#[test]
fn field_count_equals_field_names_len() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["x".into(), "y".into()])
        .build()
        .unwrap();
    assert_eq!(lang.field_count as usize, lang.field_names.len());
}

// ===========================================================================
// 27. Iterating all symbols
// ===========================================================================

#[test]
fn can_iterate_all_symbol_names() {
    let lang = language_with_symbols(
        vec!["a", "b", "c"],
        vec![terminal_visible(), terminal_hidden(), nonterminal_visible()],
    );
    let names: Vec<&str> = lang.symbol_names.iter().map(|s| s.as_str()).collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[test]
fn can_iterate_all_field_names() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["x".into(), "y".into(), "z".into()])
        .build()
        .unwrap();
    let names: Vec<&str> = lang.field_names.iter().map(|s| s.as_str()).collect();
    assert_eq!(names, vec!["x", "y", "z"]);
}

// ===========================================================================
// 28. Simulated field_id_for_name lookup
// ===========================================================================

#[test]
fn field_id_lookup_by_iterating() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["left".into(), "op".into(), "right".into()])
        .build()
        .unwrap();

    let find_field = |name: &str| -> Option<u16> {
        lang.field_names
            .iter()
            .position(|n| n == name)
            .map(|i| i as u16)
    };

    assert_eq!(find_field("left"), Some(0));
    assert_eq!(find_field("op"), Some(1));
    assert_eq!(find_field("right"), Some(2));
    assert_eq!(find_field("missing"), None);
}

// ===========================================================================
// 29. Token type
// ===========================================================================

#[test]
fn token_is_copy() {
    use adze_runtime::Token;
    let t = Token {
        kind: 1,
        start: 0,
        end: 5,
    };
    let t2 = t; // Copy
    assert_eq!(t.kind, t2.kind);
    assert_eq!(t.start, t2.start);
    assert_eq!(t.end, t2.end);
}

#[test]
fn token_debug_format() {
    use adze_runtime::Token;
    let t = Token {
        kind: 42,
        start: 10,
        end: 20,
    };
    let debug = format!("{t:?}");
    assert!(debug.contains("42"), "debug: {debug}");
}

#[test]
fn token_eq() {
    use adze_runtime::Token;
    let t1 = Token {
        kind: 1,
        start: 0,
        end: 3,
    };
    let t2 = Token {
        kind: 1,
        start: 0,
        end: 3,
    };
    let t3 = Token {
        kind: 2,
        start: 0,
        end: 3,
    };
    assert_eq!(t1, t2);
    assert_ne!(t1, t3);
}

// ===========================================================================
// 30. Builder re-use pattern (each call returns a new builder)
// ===========================================================================

#[test]
fn builder_is_consumed_on_build() {
    // Demonstrate that .build() consumes the builder (compile-time guarantee).
    // We just verify the pattern works.
    let b = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()]);
    let lang = b.build().unwrap();
    assert_eq!(lang.symbol_count, 1);
}

// ===========================================================================
// 31. Multiple builds from separate builders
// ===========================================================================

#[test]
fn multiple_builds_independent() {
    let lang1 = Language::builder()
        .version(1)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    let lang2 = Language::builder()
        .version(2)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_visible(), nonterminal_visible()])
        .build()
        .unwrap();
    assert_eq!(lang1.version, 1);
    assert_eq!(lang2.version, 2);
    assert_eq!(lang1.symbol_count, 1);
    assert_eq!(lang2.symbol_count, 2);
}

// ===========================================================================
// 32. Metadata debug
// ===========================================================================

#[test]
fn symbol_metadata_debug_format() {
    let m = terminal_visible();
    let debug = format!("{m:?}");
    assert!(debug.contains("is_terminal: true"), "debug: {debug}");
    assert!(debug.contains("is_visible: true"), "debug: {debug}");
    assert!(debug.contains("is_supertype: false"), "debug: {debug}");
}
