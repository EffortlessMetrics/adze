#![allow(clippy::needless_range_loop)]

//! Property-based and unit tests for symbol lookup in the Language API.
//!
//! Covers `symbol_for_name`, `symbol_name`, `symbol_count`, `symbol_metadata`,
//! named vs anonymous symbol lookup, determinism, and consistency invariants.

use adze_runtime::language::{Language, SymbolMetadata};
use proptest::prelude::*;

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

/// Build a language with diverse named and anonymous symbols.
fn sample_language() -> Language {
    Language::builder()
        .version(15)
        .parse_table(leak_table())
        .symbol_names(vec![
            "end".into(),         // 0 – hidden terminal (anonymous)
            "number".into(),      // 1 – visible terminal (named)
            "+".into(),           // 2 – hidden terminal (anonymous)
            "expression".into(),  // 3 – visible nonterminal (named)
            "_hidden".into(),     // 4 – hidden nonterminal (anonymous)
            "declaration".into(), // 5 – supertype (named/visible)
        ])
        .symbol_metadata(vec![
            terminal_hidden(),     // 0
            terminal_visible(),    // 1
            terminal_hidden(),     // 2
            nonterminal_visible(), // 3
            nonterminal_hidden(),  // 4
            supertype_meta(),      // 5
        ])
        .build()
        .unwrap()
}

// ===========================================================================
// 1. symbol_for_name with existing named symbol
// ===========================================================================

#[test]
fn symbol_for_name_finds_named_terminal() {
    let lang = sample_language();
    // "number" is visible (named), so is_named=true should find it at index 1
    assert_eq!(lang.symbol_for_name("number", true), Some(1));
}

#[test]
fn symbol_for_name_finds_named_nonterminal() {
    let lang = sample_language();
    // "expression" is visible nonterminal
    assert_eq!(lang.symbol_for_name("expression", true), Some(3));
}

#[test]
fn symbol_for_name_finds_named_supertype() {
    let lang = sample_language();
    // "declaration" is visible (named), supertype
    assert_eq!(lang.symbol_for_name("declaration", true), Some(5));
}

// ===========================================================================
// 2. symbol_for_name with non-existing name
// ===========================================================================

#[test]
fn symbol_for_name_returns_none_for_absent_name() {
    let lang = sample_language();
    assert_eq!(lang.symbol_for_name("nonexistent", true), None);
    assert_eq!(lang.symbol_for_name("nonexistent", false), None);
}

#[test]
fn symbol_for_name_returns_none_for_empty_string() {
    let lang = sample_language();
    assert_eq!(lang.symbol_for_name("", true), None);
    assert_eq!(lang.symbol_for_name("", false), None);
}

#[test]
fn symbol_for_name_returns_none_for_substring_match() {
    let lang = sample_language();
    // "num" is a prefix of "number" but not a full match
    assert_eq!(lang.symbol_for_name("num", true), None);
    assert_eq!(lang.symbol_for_name("express", true), None);
}

// ===========================================================================
// 3. symbol_for_name with is_named=true/false
// ===========================================================================

#[test]
fn symbol_for_name_anonymous_finds_hidden_terminal() {
    let lang = sample_language();
    // "end" is hidden terminal (anonymous) → is_named=false should find it
    assert_eq!(lang.symbol_for_name("end", false), Some(0));
    // is_named=true should NOT find it (it's not visible)
    assert_eq!(lang.symbol_for_name("end", true), None);
}

#[test]
fn symbol_for_name_anonymous_finds_hidden_nonterminal() {
    let lang = sample_language();
    // "_hidden" is hidden nonterminal → is_named=false
    assert_eq!(lang.symbol_for_name("_hidden", false), Some(4));
    assert_eq!(lang.symbol_for_name("_hidden", true), None);
}

#[test]
fn symbol_for_name_named_does_not_find_anonymous_symbol() {
    let lang = sample_language();
    // "+" is hidden terminal → only findable with is_named=false
    assert_eq!(lang.symbol_for_name("+", true), None);
    assert_eq!(lang.symbol_for_name("+", false), Some(2));
}

#[test]
fn symbol_for_name_named_does_not_find_hidden_symbol() {
    let lang = sample_language();
    // Visible symbols should not be found with is_named=false
    assert_eq!(lang.symbol_for_name("number", false), None);
    assert_eq!(lang.symbol_for_name("expression", false), None);
}

// ===========================================================================
// 4. Symbol count vs symbol names length
// ===========================================================================

#[test]
fn symbol_count_equals_names_length() {
    let lang = sample_language();
    assert_eq!(lang.symbol_count as usize, 6);
}

#[test]
fn symbol_count_matches_metadata_length() {
    let lang = sample_language();
    assert_eq!(lang.symbol_count as usize, lang.symbol_metadata.len());
}

#[test]
fn symbol_count_for_minimal_language() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_count, 1);
}

#[test]
fn symbol_count_with_no_explicit_names_uses_metadata_len() {
    // When symbol_names is not provided, builder fills with empty strings
    // matching metadata length
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

// ===========================================================================
// 5. Symbol metadata lookup by index
// ===========================================================================

#[test]
fn symbol_metadata_in_bounds_returns_correct_values() {
    let lang = sample_language();
    // Index 0: terminal_hidden
    assert!(lang.symbol_metadata[0].is_terminal);
    assert!(!lang.symbol_metadata[0].is_visible);
    // Index 1: terminal_visible
    assert!(lang.symbol_metadata[1].is_terminal);
    assert!(lang.symbol_metadata[1].is_visible);
    // Index 3: nonterminal_visible
    assert!(!lang.symbol_metadata[3].is_terminal);
    assert!(lang.symbol_metadata[3].is_visible);
    // Index 5: supertype
    assert!(lang.symbol_metadata[5].is_supertype);
}

#[test]
fn symbol_metadata_length_consistent_with_symbol_count() {
    let lang = sample_language();
    assert_eq!(lang.symbol_metadata.len(), lang.symbol_count as usize);
}

// ===========================================================================
// 6. Symbol name lookup consistency
// ===========================================================================

#[test]
fn symbol_name_roundtrips_with_symbol_for_name_named() {
    let lang = sample_language();
    // For every visible (named) symbol, symbol_for_name should return its index
    for i in 0..lang.symbol_count as usize {
        if let Some(name) = lang.symbol_name(i as u16) {
            let is_visible = lang.symbol_metadata[i].is_visible;
            let found = lang.symbol_for_name(name, is_visible);
            assert_eq!(
                found,
                Some(i as u16),
                "roundtrip failed for symbol {i} ({name})"
            );
        }
    }
}

#[test]
fn symbol_name_out_of_range_returns_none() {
    let lang = sample_language();
    assert_eq!(lang.symbol_name(100), None);
    assert_eq!(lang.symbol_name(u16::MAX), None);
}

#[test]
fn symbol_name_returns_expected_strings() {
    let lang = sample_language();
    assert_eq!(lang.symbol_name(0), Some("end"));
    assert_eq!(lang.symbol_name(1), Some("number"));
    assert_eq!(lang.symbol_name(2), Some("+"));
    assert_eq!(lang.symbol_name(3), Some("expression"));
    assert_eq!(lang.symbol_name(4), Some("_hidden"));
    assert_eq!(lang.symbol_name(5), Some("declaration"));
}

// ===========================================================================
// 7. Anonymous vs named symbol lookup
// ===========================================================================

#[test]
fn all_visible_symbols_found_with_is_named_true() {
    let lang = sample_language();
    let visible_indices: Vec<u16> = (0..lang.symbol_count as usize)
        .filter(|&i| lang.symbol_metadata[i].is_visible)
        .map(|i| i as u16)
        .collect();
    for idx in &visible_indices {
        let name = lang.symbol_name(*idx).unwrap();
        assert_eq!(
            lang.symbol_for_name(name, true),
            Some(*idx),
            "visible symbol {name} at index {idx} not found with is_named=true"
        );
    }
}

#[test]
fn all_hidden_symbols_found_with_is_named_false() {
    let lang = sample_language();
    let hidden_indices: Vec<u16> = (0..lang.symbol_count as usize)
        .filter(|&i| !lang.symbol_metadata[i].is_visible)
        .map(|i| i as u16)
        .collect();
    for idx in &hidden_indices {
        let name = lang.symbol_name(*idx).unwrap();
        assert_eq!(
            lang.symbol_for_name(name, false),
            Some(*idx),
            "hidden symbol {name} at index {idx} not found with is_named=false"
        );
    }
}

#[test]
fn visible_and_hidden_symbols_are_disjoint_for_same_name() {
    let lang = sample_language();
    // For each symbol, exactly one of is_named=true/false should return Some
    for i in 0..lang.symbol_count as usize {
        let name = lang.symbol_name(i as u16).unwrap();
        let found_named = lang.symbol_for_name(name, true);
        let found_anon = lang.symbol_for_name(name, false);
        // At least one should find it, and they should be mutually exclusive
        // (unless there are duplicates with different visibility)
        assert!(
            found_named.is_some() || found_anon.is_some(),
            "symbol {name} not found by either lookup"
        );
    }
}

// ===========================================================================
// 8. Symbol lookup determinism
// ===========================================================================

#[test]
fn symbol_for_name_is_deterministic_across_calls() {
    let lang = sample_language();
    for _ in 0..10 {
        assert_eq!(lang.symbol_for_name("number", true), Some(1));
        assert_eq!(lang.symbol_for_name("+", false), Some(2));
        assert_eq!(lang.symbol_for_name("expression", true), Some(3));
        assert_eq!(lang.symbol_for_name("nonexistent", true), None);
    }
}

#[test]
fn symbol_for_name_deterministic_across_identical_languages() {
    let lang1 = sample_language();
    let lang2 = sample_language();
    for i in 0..lang1.symbol_count as usize {
        let name = lang1.symbol_name(i as u16).unwrap();
        let is_vis = lang1.symbol_metadata[i].is_visible;
        assert_eq!(
            lang1.symbol_for_name(name, is_vis),
            lang2.symbol_for_name(name, is_vis),
            "determinism broken for symbol {name}"
        );
    }
}

#[test]
fn symbol_name_deterministic_across_calls() {
    let lang = sample_language();
    for i in 0..lang.symbol_count as u16 {
        let first = lang.symbol_name(i);
        let second = lang.symbol_name(i);
        assert_eq!(first, second, "symbol_name not deterministic for index {i}");
    }
}

// ===========================================================================
// 9. Proptest: property-based symbol lookup tests
// ===========================================================================

proptest! {
    #[test]
    fn proptest_random_name_lookup_never_panics(name in "[a-z_]{0,20}") {
        let lang = sample_language();
        // Should never panic regardless of input
        let _ = lang.symbol_for_name(&name, true);
        let _ = lang.symbol_for_name(&name, false);
    }

    #[test]
    fn proptest_symbol_name_out_of_bounds_returns_none(id in 6u16..=u16::MAX) {
        let lang = sample_language();
        assert_eq!(lang.symbol_name(id), None);
    }

    #[test]
    fn proptest_symbol_for_name_returns_valid_index(idx in 0usize..6) {
        let lang = sample_language();
        let name = lang.symbol_name(idx as u16).unwrap();
        let is_visible = lang.symbol_metadata[idx].is_visible;
        let found = lang.symbol_for_name(name, is_visible).unwrap();
        prop_assert!((found as usize) < lang.symbol_count as usize);
    }

    #[test]
    fn proptest_symbol_for_name_wrong_visibility_returns_none(idx in 0usize..6) {
        let lang = sample_language();
        let name = lang.symbol_name(idx as u16).unwrap();
        let is_visible = lang.symbol_metadata[idx].is_visible;
        // Flip visibility: should NOT find the symbol
        let found = lang.symbol_for_name(name, !is_visible);
        prop_assert_eq!(found, None);
    }

    #[test]
    fn proptest_dynamic_language_symbol_count_matches(
        count in 1usize..=10,
    ) {
        let names: Vec<String> = (0..count).map(|i| format!("sym_{i}")).collect();
        let meta: Vec<SymbolMetadata> = (0..count)
            .map(|i| if i % 2 == 0 { terminal_visible() } else { terminal_hidden() })
            .collect();
        let lang = Language::builder()
            .parse_table(leak_table())
            .symbol_names(names)
            .symbol_metadata(meta)
            .build()
            .unwrap();
        prop_assert_eq!(lang.symbol_count as usize, count);
        prop_assert_eq!(lang.symbol_metadata.len(), count);
    }

    #[test]
    fn proptest_dynamic_language_roundtrip(
        count in 1usize..=8,
    ) {
        let names: Vec<String> = (0..count).map(|i| format!("sym_{i}")).collect();
        let meta: Vec<SymbolMetadata> = (0..count)
            .map(|i| if i % 2 == 0 { terminal_visible() } else { terminal_hidden() })
            .collect();
        let lang = Language::builder()
            .parse_table(leak_table())
            .symbol_names(names.clone())
            .symbol_metadata(meta.clone())
            .build()
            .unwrap();
        for i in 0..count {
            let name = lang.symbol_name(i as u16).unwrap();
            prop_assert_eq!(name, &names[i]);
            let is_visible = meta[i].is_visible;
            let found = lang.symbol_for_name(name, is_visible);
            prop_assert_eq!(found, Some(i as u16));
        }
    }
}
