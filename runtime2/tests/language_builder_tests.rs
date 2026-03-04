//! Tests for the LanguageBuilder API.

use adze_runtime::language::{Language, SymbolMetadata};

fn eof_meta() -> SymbolMetadata {
    SymbolMetadata {
        is_terminal: true,
        is_visible: false,
        is_supertype: false,
    }
}

fn named_meta() -> SymbolMetadata {
    SymbolMetadata {
        is_terminal: false,
        is_visible: true,
        is_supertype: false,
    }
}

fn leak_parse_table() -> &'static adze_glr_core::ParseTable {
    let table = adze_glr_core::ParseTable::default();
    Box::leak(Box::new(table))
}

#[test]
fn builder_builds_with_required_fields() {
    let lang = Language::builder()
        .parse_table(leak_parse_table())
        .symbol_metadata(vec![eof_meta(), named_meta()])
        .build();
    assert!(lang.is_ok());
}

#[test]
fn builder_with_version() {
    let lang = Language::builder()
        .version(14)
        .parse_table(leak_parse_table())
        .symbol_metadata(vec![eof_meta(), named_meta()])
        .build();
    assert!(lang.is_ok());
}

#[test]
fn builder_missing_parse_table_errors() {
    let lang = Language::builder()
        .symbol_metadata(vec![eof_meta()])
        .build();
    assert!(lang.is_err());
}

#[test]
fn builder_missing_metadata_errors() {
    let lang = Language::builder().parse_table(leak_parse_table()).build();
    assert!(lang.is_err());
}

#[test]
fn builder_symbol_names() {
    let lang = Language::builder()
        .parse_table(leak_parse_table())
        .symbol_names(vec!["end".into(), "source".into()])
        .symbol_metadata(vec![eof_meta(), named_meta()])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_name(0), Some("end"));
    assert_eq!(lang.symbol_name(1), Some("source"));
}

#[test]
fn builder_field_names() {
    let lang = Language::builder()
        .parse_table(leak_parse_table())
        .symbol_names(vec!["end".into(), "expr".into()])
        .symbol_metadata(vec![eof_meta(), named_meta()])
        .field_names(vec!["left".into(), "right".into()])
        .build()
        .unwrap();
    assert_eq!(lang.field_name(0), Some("left"));
    assert_eq!(lang.field_name(1), Some("right"));
}

#[test]
fn builder_is_visible() {
    let lang = Language::builder()
        .parse_table(leak_parse_table())
        .symbol_metadata(vec![eof_meta(), named_meta()])
        .build()
        .unwrap();
    assert!(!lang.is_visible(0));
    assert!(lang.is_visible(1));
}

#[test]
fn builder_missing_symbol_name_returns_none() {
    let lang = Language::builder()
        .parse_table(leak_parse_table())
        .symbol_metadata(vec![eof_meta()])
        .build()
        .unwrap();
    assert!(lang.symbol_name(99).is_none());
}

#[test]
fn builder_missing_field_name_returns_none() {
    let lang = Language::builder()
        .parse_table(leak_parse_table())
        .symbol_metadata(vec![eof_meta()])
        .build()
        .unwrap();
    assert!(lang.field_name(99).is_none());
}

#[test]
fn builder_max_alias_sequence_length() {
    let lang = Language::builder()
        .max_alias_sequence_length(5)
        .parse_table(leak_parse_table())
        .symbol_metadata(vec![eof_meta(), named_meta()])
        .build();
    assert!(lang.is_ok());
}

#[test]
fn language_debug_impl() {
    let lang = Language::builder()
        .parse_table(leak_parse_table())
        .symbol_metadata(vec![eof_meta()])
        .build()
        .unwrap();
    let debug = format!("{lang:?}");
    assert!(!debug.is_empty());
}

#[test]
fn builder_is_terminal() {
    let lang = Language::builder()
        .parse_table(leak_parse_table())
        .symbol_metadata(vec![eof_meta(), named_meta()])
        .build()
        .unwrap();
    assert!(lang.is_terminal(0));
    assert!(!lang.is_terminal(1));
}
