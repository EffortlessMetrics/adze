#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for Language metadata access in adze-runtime.
//!
//! Covers symbol_count, symbol_name, symbol_type/metadata, field_count,
//! field_name_for_id, symbol_for_name, invalid ID handling, and version().

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

/// Build a language with the given symbol names, metadata, and field names.
fn build_language(
    version: u32,
    sym_names: Vec<&str>,
    sym_meta: Vec<SymbolMetadata>,
    field_names: Vec<&str>,
) -> Language {
    Language::builder()
        .version(version)
        .parse_table(leak_table())
        .symbol_names(sym_names.into_iter().map(String::from).collect())
        .symbol_metadata(sym_meta)
        .field_names(field_names.into_iter().map(String::from).collect())
        .build()
        .unwrap()
}

/// A richer language fixture for many tests.
fn rich_language() -> Language {
    build_language(
        15,
        vec!["END", "number", "+", "expression", "statement", "_type"],
        vec![
            terminal_hidden(),      // 0: END – hidden terminal
            terminal_visible(),     // 1: number – visible terminal
            terminal_hidden(),      // 2: "+" – hidden terminal (punctuation)
            nonterminal_visible(),  // 3: expression – visible non-terminal
            nonterminal_hidden(),   // 4: statement – hidden non-terminal
            supertype_meta(),       // 5: _type – supertype
        ],
        vec!["left", "right", "body"],
    )
}

// ===========================================================================
// 1. symbol_count
// ===========================================================================

#[test]
fn symbol_count_matches_configured_symbols() {
    let lang = rich_language();
    assert_eq!(lang.symbol_count, 6);
}

#[test]
fn symbol_count_single_symbol() {
    let lang = build_language(1, vec!["x"], vec![terminal_visible()], vec![]);
    assert_eq!(lang.symbol_count, 1);
}

#[test]
fn symbol_count_zero_when_no_symbols() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 0);
}

// ===========================================================================
// 2. symbol_name(id)
// ===========================================================================

#[test]
fn symbol_name_returns_correct_names() {
    let lang = rich_language();
    assert_eq!(lang.symbol_name(0), Some("END"));
    assert_eq!(lang.symbol_name(1), Some("number"));
    assert_eq!(lang.symbol_name(2), Some("+"));
    assert_eq!(lang.symbol_name(3), Some("expression"));
    assert_eq!(lang.symbol_name(4), Some("statement"));
    assert_eq!(lang.symbol_name(5), Some("_type"));
}

#[test]
fn symbol_name_none_for_out_of_bounds() {
    let lang = rich_language();
    assert_eq!(lang.symbol_name(6), None);
    assert_eq!(lang.symbol_name(100), None);
    assert_eq!(lang.symbol_name(u16::MAX), None);
}

#[test]
fn symbol_name_all_ids_in_range() {
    let lang = rich_language();
    for i in 0..lang.symbol_count {
        assert!(
            lang.symbol_name(i as u16).is_some(),
            "symbol_name({i}) should be Some"
        );
    }
}

// ===========================================================================
// 3. symbol_type / symbol_metadata
// ===========================================================================

#[test]
fn is_terminal_returns_correct_values() {
    let lang = rich_language();
    assert!(lang.is_terminal(0));  // END
    assert!(lang.is_terminal(1));  // number
    assert!(lang.is_terminal(2));  // +
    assert!(!lang.is_terminal(3)); // expression (non-terminal)
    assert!(!lang.is_terminal(4)); // statement (non-terminal)
    assert!(!lang.is_terminal(5)); // _type (supertype, non-terminal)
}

#[test]
fn is_visible_returns_correct_values() {
    let lang = rich_language();
    assert!(!lang.is_visible(0)); // END – hidden
    assert!(lang.is_visible(1));  // number – visible
    assert!(!lang.is_visible(2)); // + – hidden
    assert!(lang.is_visible(3));  // expression – visible
    assert!(!lang.is_visible(4)); // statement – hidden
    assert!(lang.is_visible(5));  // _type – visible (supertype)
}

#[test]
fn is_terminal_false_for_out_of_bounds() {
    let lang = rich_language();
    assert!(!lang.is_terminal(99));
    assert!(!lang.is_terminal(u16::MAX));
}

#[test]
fn is_visible_false_for_out_of_bounds() {
    let lang = rich_language();
    assert!(!lang.is_visible(99));
    assert!(!lang.is_visible(u16::MAX));
}

#[test]
fn symbol_metadata_supertype_flag() {
    let lang = rich_language();
    assert!(lang.symbol_metadata[5].is_supertype);
    for i in 0..5 {
        assert!(!lang.symbol_metadata[i].is_supertype);
    }
}

#[test]
fn symbol_metadata_vec_length_matches_symbol_count() {
    let lang = rich_language();
    assert_eq!(lang.symbol_metadata.len(), lang.symbol_count as usize);
}

// ===========================================================================
// 4. field_count
// ===========================================================================

#[test]
fn field_count_matches_configured_fields() {
    let lang = rich_language();
    assert_eq!(lang.field_count, 3);
}

#[test]
fn field_count_zero_when_no_fields() {
    let lang = build_language(1, vec!["x"], vec![terminal_visible()], vec![]);
    assert_eq!(lang.field_count, 0);
}

#[test]
fn field_count_defaults_to_zero_without_field_names() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.field_count, 0);
}

// ===========================================================================
// 5. field_name_for_id(id)
// ===========================================================================

#[test]
fn field_name_returns_correct_names() {
    let lang = rich_language();
    assert_eq!(lang.field_name(0), Some("left"));
    assert_eq!(lang.field_name(1), Some("right"));
    assert_eq!(lang.field_name(2), Some("body"));
}

#[test]
fn field_name_none_for_out_of_bounds() {
    let lang = rich_language();
    assert_eq!(lang.field_name(3), None);
    assert_eq!(lang.field_name(100), None);
    assert_eq!(lang.field_name(u16::MAX), None);
}

#[test]
fn field_name_all_ids_in_range() {
    let lang = rich_language();
    for i in 0..lang.field_count {
        assert!(
            lang.field_name(i as u16).is_some(),
            "field_name({i}) should be Some"
        );
    }
}

// ===========================================================================
// 6. symbol_for_name(name, is_named)
// ===========================================================================

#[test]
fn symbol_for_name_finds_visible_terminal() {
    let lang = rich_language();
    // "number" is terminal + visible → is_named = true
    assert_eq!(lang.symbol_for_name("number", true), Some(1));
}

#[test]
fn symbol_for_name_finds_hidden_terminal() {
    let lang = rich_language();
    // "END" is terminal + hidden → is_named = false
    assert_eq!(lang.symbol_for_name("END", false), Some(0));
}

#[test]
fn symbol_for_name_finds_visible_nonterminal() {
    let lang = rich_language();
    // "expression" is non-terminal + visible → is_named = true
    assert_eq!(lang.symbol_for_name("expression", true), Some(3));
}

#[test]
fn symbol_for_name_finds_hidden_nonterminal() {
    let lang = rich_language();
    // "statement" is non-terminal + hidden → is_named = false
    assert_eq!(lang.symbol_for_name("statement", false), Some(4));
}

#[test]
fn symbol_for_name_finds_supertype() {
    let lang = rich_language();
    // "_type" is supertype (visible) → is_named = true
    assert_eq!(lang.symbol_for_name("_type", true), Some(5));
}

#[test]
fn symbol_for_name_none_for_wrong_visibility() {
    let lang = rich_language();
    // "number" is visible; searching with is_named=false should fail
    assert_eq!(lang.symbol_for_name("number", false), None);
    // "END" is hidden; searching with is_named=true should fail
    assert_eq!(lang.symbol_for_name("END", true), None);
}

#[test]
fn symbol_for_name_none_for_nonexistent_name() {
    let lang = rich_language();
    assert_eq!(lang.symbol_for_name("nonexistent", true), None);
    assert_eq!(lang.symbol_for_name("nonexistent", false), None);
}

#[test]
fn symbol_for_name_empty_string() {
    let lang = rich_language();
    assert_eq!(lang.symbol_for_name("", true), None);
    assert_eq!(lang.symbol_for_name("", false), None);
}

// ===========================================================================
// 7. Invalid symbol ID handling
// ===========================================================================

#[test]
fn invalid_symbol_id_name_returns_none() {
    let lang = rich_language();
    assert!(lang.symbol_name(255).is_none());
}

#[test]
fn invalid_symbol_id_terminal_returns_false() {
    let lang = rich_language();
    assert!(!lang.is_terminal(255));
}

#[test]
fn invalid_symbol_id_visible_returns_false() {
    let lang = rich_language();
    assert!(!lang.is_visible(255));
}

#[test]
fn boundary_symbol_id_just_past_end() {
    let lang = rich_language();
    let boundary = lang.symbol_count as u16;
    assert_eq!(lang.symbol_name(boundary), None);
    assert!(!lang.is_terminal(boundary));
    assert!(!lang.is_visible(boundary));
}

// ===========================================================================
// 8. Invalid field ID handling
// ===========================================================================

#[test]
fn invalid_field_id_returns_none() {
    let lang = rich_language();
    assert!(lang.field_name(255).is_none());
}

#[test]
fn boundary_field_id_just_past_end() {
    let lang = rich_language();
    let boundary = lang.field_count as u16;
    assert_eq!(lang.field_name(boundary), None);
}

#[test]
fn field_name_on_language_with_no_fields() {
    let lang = build_language(1, vec!["x"], vec![terminal_visible()], vec![]);
    assert_eq!(lang.field_name(0), None);
}

// ===========================================================================
// 9. version() returns correct ABI version
// ===========================================================================

#[test]
fn version_matches_configured_value() {
    let lang = rich_language();
    assert_eq!(lang.version, 15);
}

#[test]
fn version_defaults_to_zero() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.version, 0);
}

#[test]
fn version_custom_value() {
    let lang = build_language(42, vec!["a"], vec![terminal_visible()], vec![]);
    assert_eq!(lang.version, 42);
}
