//! Comprehensive tests for the Language type and LanguageBuilder.

use adze_runtime::language::RuntimeParseTable;
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

fn supertype() -> SymbolMetadata {
    SymbolMetadata {
        is_terminal: false,
        is_visible: true,
        is_supertype: true,
    }
}

fn leak_table() -> &'static RuntimeParseTable {
    Box::leak(Box::new(RuntimeParseTable::default()))
}

/// Minimal valid builder: parse_table + symbol_metadata.
fn minimal_language() -> Language {
    Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap()
}

// ===========================================================================
// 1. Builder – success with all required fields
// ===========================================================================

#[test]
fn build_with_only_required_fields() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build();
    assert!(lang.is_ok());
}

#[test]
fn build_with_all_optional_fields() {
    let lang = Language::builder()
        .version(15)
        .max_alias_sequence_length(3)
        .parse_table(leak_table())
        .symbol_names(vec!["EOF".into(), "expr".into()])
        .symbol_metadata(vec![terminal_hidden(), nonterminal_visible()])
        .field_names(vec!["left".into(), "right".into()])
        .build();
    assert!(lang.is_ok());
}

// ===========================================================================
// 2. Builder – missing required fields
// ===========================================================================

#[test]
fn build_fails_without_parse_table() {
    let result = Language::builder()
        .symbol_metadata(vec![terminal_hidden()])
        .build();
    assert_eq!(result.unwrap_err(), "missing parse table");
}

#[test]
fn build_fails_without_symbol_metadata() {
    let result = Language::builder().parse_table(leak_table()).build();
    assert_eq!(result.unwrap_err(), "missing symbol metadata");
}

#[test]
fn build_fails_without_both_required() {
    let result = Language::builder().build();
    assert!(result.is_err());
}

// ===========================================================================
// 3. Language queries – symbol_name
// ===========================================================================

#[test]
fn symbol_name_returns_correct_name() {
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
fn symbol_name_at_exact_boundary() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec!["a".into(), "b".into()])
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .build()
        .unwrap();

    assert_eq!(lang.symbol_name(1), Some("b"));
    assert_eq!(lang.symbol_name(2), None);
}

// ===========================================================================
// 4. Language queries – field_name
// ===========================================================================

#[test]
fn field_name_returns_correct_name() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["condition".into(), "body".into()])
        .build()
        .unwrap();

    assert_eq!(lang.field_name(0), Some("condition"));
    assert_eq!(lang.field_name(1), Some("body"));
}

#[test]
fn field_name_out_of_bounds_returns_none() {
    let lang = minimal_language();
    assert_eq!(lang.field_name(0), None);
    assert_eq!(lang.field_name(100), None);
}

// ===========================================================================
// 5. Language queries – is_terminal / is_visible
// ===========================================================================

#[test]
fn is_terminal_true_for_terminal() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_visible(), nonterminal_visible()])
        .build()
        .unwrap();

    assert!(lang.is_terminal(0));
    assert!(!lang.is_terminal(1));
}

#[test]
fn is_terminal_out_of_bounds_returns_false() {
    let lang = minimal_language();
    assert!(!lang.is_terminal(999));
}

#[test]
fn is_visible_true_for_visible() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden(), nonterminal_visible()])
        .build()
        .unwrap();

    assert!(!lang.is_visible(0));
    assert!(lang.is_visible(1));
}

#[test]
fn is_visible_out_of_bounds_returns_false() {
    let lang = minimal_language();
    assert!(!lang.is_visible(999));
}

// ===========================================================================
// 6. Language version
// ===========================================================================

#[test]
fn version_defaults_to_zero() {
    let lang = minimal_language();
    assert_eq!(lang.version, 0);
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
fn version_can_be_arbitrary_value() {
    let lang = Language::builder()
        .version(42)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.version, 42);
}

// ===========================================================================
// 7. Symbol count / field count
// ===========================================================================

#[test]
fn symbol_count_matches_metadata_len() {
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
fn symbol_count_with_explicit_names() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec!["a".into(), "b".into()])
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 2);
}

#[test]
fn field_count_zero_when_no_fields() {
    let lang = minimal_language();
    assert_eq!(lang.field_count, 0);
}

#[test]
fn field_count_matches_field_names_len() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["x".into(), "y".into(), "z".into()])
        .build()
        .unwrap();
    assert_eq!(lang.field_count, 3);
}

// ===========================================================================
// 8. Empty symbol names defaults
// ===========================================================================

#[test]
fn symbol_names_default_to_empty_strings() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .build()
        .unwrap();

    // When no symbol_names provided, defaults to empty strings
    assert_eq!(lang.symbol_name(0), Some(""));
    assert_eq!(lang.symbol_name(1), Some(""));
    assert_eq!(lang.symbol_count, 2);
}

#[test]
fn field_names_default_to_empty_vec() {
    let lang = minimal_language();
    assert_eq!(lang.field_count, 0);
    assert_eq!(lang.field_name(0), None);
}

// ===========================================================================
// 9. Multiple metadata entries / mixed symbol types
// ===========================================================================

#[test]
fn mixed_symbol_metadata() {
    let meta = vec![
        terminal_hidden(),     // 0: EOF
        terminal_visible(),    // 1: number
        nonterminal_visible(), // 2: expression
        nonterminal_hidden(),  // 3: _rule
        supertype(),           // 4: _statement
    ];
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(meta)
        .build()
        .unwrap();

    assert!(lang.is_terminal(0));
    assert!(!lang.is_visible(0));

    assert!(lang.is_terminal(1));
    assert!(lang.is_visible(1));

    assert!(!lang.is_terminal(2));
    assert!(lang.is_visible(2));

    assert!(!lang.is_terminal(3));
    assert!(!lang.is_visible(3));

    assert!(!lang.is_terminal(4));
    assert!(lang.is_visible(4));
}

#[test]
fn supertype_metadata_stored_correctly() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![supertype()])
        .build()
        .unwrap();
    assert!(lang.symbol_metadata[0].is_supertype);
}

#[test]
fn single_symbol_metadata() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_visible()])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 1);
    assert!(lang.is_terminal(0));
    assert!(lang.is_visible(0));
}

// ===========================================================================
// 10. Debug formatting
// ===========================================================================

#[test]
fn debug_format_contains_version() {
    let lang = Language::builder()
        .version(15)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    let debug = format!("{lang:?}");
    assert!(debug.contains("version: 15"), "debug: {debug}");
}

#[test]
fn debug_format_contains_symbol_count() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .build()
        .unwrap();
    let debug = format!("{lang:?}");
    assert!(debug.contains("symbol_count: 2"), "debug: {debug}");
}

#[test]
fn debug_format_contains_field_count() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["f1".into()])
        .build()
        .unwrap();
    let debug = format!("{lang:?}");
    assert!(debug.contains("field_count: 1"), "debug: {debug}");
}

#[test]
fn debug_format_contains_symbol_names() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec!["foo".into()])
        .symbol_metadata(vec![terminal_visible()])
        .build()
        .unwrap();
    let debug = format!("{lang:?}");
    assert!(debug.contains("foo"), "debug: {debug}");
}

#[test]
fn debug_format_nonempty() {
    let lang = minimal_language();
    let debug = format!("{lang:?}");
    assert!(!debug.is_empty());
    assert!(debug.starts_with("Language"));
}

// ===========================================================================
// 11. max_alias_sequence_length
// ===========================================================================

#[test]
fn max_alias_sequence_length_defaults_to_zero() {
    let lang = minimal_language();
    assert_eq!(lang.max_alias_sequence_length, 0);
}

#[test]
fn max_alias_sequence_length_set_via_builder() {
    let lang = Language::builder()
        .max_alias_sequence_length(10)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.max_alias_sequence_length, 10);
}

// ===========================================================================
// 12. Clone behavior
// ===========================================================================

#[test]
fn clone_preserves_version() {
    let lang = Language::builder()
        .version(7)
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    let cloned = lang.clone();
    assert_eq!(cloned.version, 7);
}

#[test]
fn clone_preserves_symbol_names() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec!["alpha".into(), "beta".into()])
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .build()
        .unwrap();
    let cloned = lang.clone();
    assert_eq!(cloned.symbol_name(0), Some("alpha"));
    assert_eq!(cloned.symbol_name(1), Some("beta"));
}

#[test]
fn clone_preserves_counts() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden(), terminal_visible()])
        .field_names(vec!["x".into()])
        .build()
        .unwrap();
    let cloned = lang.clone();
    assert_eq!(cloned.symbol_count, 2);
    assert_eq!(cloned.field_count, 1);
}

// ===========================================================================
// 13. Builder method chaining order independence
// ===========================================================================

#[test]
fn builder_order_does_not_matter() {
    // Set fields in reverse order of the "normal" pattern
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
    assert_eq!(lang.symbol_count, 1);
    assert_eq!(lang.field_count, 1);
}

// ===========================================================================
// 14. Edge cases
// ===========================================================================

#[test]
fn empty_metadata_builds_successfully() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 0);
}

#[test]
fn symbol_name_with_unicode() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec!["λ".into(), "→".into()])
        .symbol_metadata(vec![terminal_visible(), terminal_visible()])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_name(0), Some("λ"));
    assert_eq!(lang.symbol_name(1), Some("→"));
}

#[test]
fn field_name_with_empty_string() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .field_names(vec!["".into()])
        .build()
        .unwrap();
    assert_eq!(lang.field_name(0), Some(""));
    assert_eq!(lang.field_count, 1);
}

#[test]
fn many_symbols() {
    let n = 256;
    let names: Vec<String> = (0..n).map(|i| format!("sym_{i}")).collect();
    let meta: Vec<SymbolMetadata> = (0..n).map(|_| terminal_visible()).collect();
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(names)
        .symbol_metadata(meta)
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 256);
    assert_eq!(lang.symbol_name(0), Some("sym_0"));
    assert_eq!(lang.symbol_name(255), Some("sym_255"));
}
