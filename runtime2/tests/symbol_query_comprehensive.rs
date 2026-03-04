#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for symbol querying in the Language API.
//!
//! Covers symbol name lookup by ID, field name lookup by ID, symbol lookup by
//! name, symbol/field count accuracy, terminal vs nonterminal classification,
//! visible vs hidden symbols, supertype symbols, out-of-range lookups, and
//! multiple symbol queries.

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

/// Build a small language with a handful of diverse symbols and fields.
fn sample_language() -> Language {
    Language::builder()
        .version(15)
        .parse_table(leak_table())
        .symbol_names(vec![
            "end".into(),         // 0 – hidden terminal (e.g. EOF)
            "number".into(),      // 1 – visible terminal
            "+".into(),           // 2 – hidden terminal (punctuation)
            "expression".into(),  // 3 – visible nonterminal
            "_statement".into(),  // 4 – hidden nonterminal
            "declaration".into(), // 5 – supertype
        ])
        .symbol_metadata(vec![
            terminal_hidden(),     // 0
            terminal_visible(),    // 1
            terminal_hidden(),     // 2
            nonterminal_visible(), // 3
            nonterminal_hidden(),  // 4
            supertype_meta(),      // 5
        ])
        .field_names(vec!["left".into(), "operator".into(), "right".into()])
        .build()
        .unwrap()
}

// ===========================================================================
// 1. Symbol name lookup by ID
// ===========================================================================

#[test]
fn symbol_name_first_symbol() {
    let lang = sample_language();
    assert_eq!(lang.symbol_name(0), Some("end"));
}

#[test]
fn symbol_name_visible_terminal() {
    let lang = sample_language();
    assert_eq!(lang.symbol_name(1), Some("number"));
}

#[test]
fn symbol_name_punctuation_symbol() {
    let lang = sample_language();
    assert_eq!(lang.symbol_name(2), Some("+"));
}

#[test]
fn symbol_name_nonterminal() {
    let lang = sample_language();
    assert_eq!(lang.symbol_name(3), Some("expression"));
}

#[test]
fn symbol_name_hidden_nonterminal() {
    let lang = sample_language();
    assert_eq!(lang.symbol_name(4), Some("_statement"));
}

#[test]
fn symbol_name_supertype() {
    let lang = sample_language();
    assert_eq!(lang.symbol_name(5), Some("declaration"));
}

#[test]
fn symbol_name_out_of_range() {
    let lang = sample_language();
    assert_eq!(lang.symbol_name(6), None);
}

#[test]
fn symbol_name_far_out_of_range() {
    let lang = sample_language();
    assert_eq!(lang.symbol_name(u16::MAX), None);
}

// ===========================================================================
// 2. Field name lookup by ID
// ===========================================================================

#[test]
fn field_name_first() {
    let lang = sample_language();
    assert_eq!(lang.field_name(0), Some("left"));
}

#[test]
fn field_name_last() {
    let lang = sample_language();
    assert_eq!(lang.field_name(2), Some("right"));
}

#[test]
fn field_name_out_of_range() {
    let lang = sample_language();
    assert_eq!(lang.field_name(3), None);
}

#[test]
fn field_name_far_out_of_range() {
    let lang = sample_language();
    assert_eq!(lang.field_name(u16::MAX), None);
}

// ===========================================================================
// 3. Symbol lookup by name (symbol_for_name)
// ===========================================================================

#[test]
fn symbol_for_name_visible_terminal() {
    let lang = sample_language();
    assert_eq!(lang.symbol_for_name("number", true), Some(1));
}

#[test]
fn symbol_for_name_hidden_terminal() {
    let lang = sample_language();
    // "+" is hidden, so is_named=false should match.
    assert_eq!(lang.symbol_for_name("+", false), Some(2));
}

#[test]
fn symbol_for_name_visible_nonterminal() {
    let lang = sample_language();
    assert_eq!(lang.symbol_for_name("expression", true), Some(3));
}

#[test]
fn symbol_for_name_hidden_nonterminal() {
    let lang = sample_language();
    assert_eq!(lang.symbol_for_name("_statement", false), Some(4));
}

#[test]
fn symbol_for_name_supertype_is_named() {
    let lang = sample_language();
    // Supertypes are visible, so is_named=true should match.
    assert_eq!(lang.symbol_for_name("declaration", true), Some(5));
}

#[test]
fn symbol_for_name_wrong_named_flag() {
    let lang = sample_language();
    // "number" is visible; asking for anonymous should fail.
    assert_eq!(lang.symbol_for_name("number", false), None);
}

#[test]
fn symbol_for_name_nonexistent() {
    let lang = sample_language();
    assert_eq!(lang.symbol_for_name("nonexistent", true), None);
    assert_eq!(lang.symbol_for_name("nonexistent", false), None);
}

// ===========================================================================
// 4. Symbol and field count accuracy
// ===========================================================================

#[test]
fn symbol_count_matches_names() {
    let lang = sample_language();
    assert_eq!(lang.symbol_count, 6);
    assert_eq!(lang.symbol_count as usize, lang.symbol_names.len());
}

#[test]
fn field_count_matches_names() {
    let lang = sample_language();
    assert_eq!(lang.field_count, 3);
    assert_eq!(lang.field_count as usize, lang.field_names.len());
}

#[test]
fn zero_field_language() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_names(vec!["end".into()])
        .symbol_metadata(vec![terminal_hidden()])
        .build()
        .unwrap();
    assert_eq!(lang.field_count, 0);
    assert_eq!(lang.field_name(0), None);
}

// ===========================================================================
// 5. Terminal vs nonterminal classification
// ===========================================================================

#[test]
fn is_terminal_true_for_terminals() {
    let lang = sample_language();
    assert!(lang.is_terminal(0)); // end
    assert!(lang.is_terminal(1)); // number
    assert!(lang.is_terminal(2)); // +
}

#[test]
fn is_terminal_false_for_nonterminals() {
    let lang = sample_language();
    assert!(!lang.is_terminal(3)); // expression
    assert!(!lang.is_terminal(4)); // _statement
    assert!(!lang.is_terminal(5)); // declaration (supertype, nonterminal)
}

#[test]
fn is_terminal_out_of_range() {
    let lang = sample_language();
    assert!(!lang.is_terminal(100));
}

// ===========================================================================
// 6. Visible vs hidden symbols
// ===========================================================================

#[test]
fn is_visible_true_for_visible() {
    let lang = sample_language();
    assert!(lang.is_visible(1)); // number
    assert!(lang.is_visible(3)); // expression
    assert!(lang.is_visible(5)); // declaration (supertype)
}

#[test]
fn is_visible_false_for_hidden() {
    let lang = sample_language();
    assert!(!lang.is_visible(0)); // end
    assert!(!lang.is_visible(2)); // +
    assert!(!lang.is_visible(4)); // _statement
}

#[test]
fn is_visible_out_of_range() {
    let lang = sample_language();
    assert!(!lang.is_visible(99));
}

// ===========================================================================
// 7. Supertype symbols
// ===========================================================================

#[test]
fn supertype_flag_set_correctly() {
    let lang = sample_language();
    assert!(lang.symbol_metadata[5].is_supertype);
}

#[test]
fn non_supertype_flag_clear() {
    let lang = sample_language();
    for i in 0..5 {
        assert!(
            !lang.symbol_metadata[i].is_supertype,
            "symbol {} should not be a supertype",
            i
        );
    }
}

// ===========================================================================
// 8. Out-of-range lookups (additional edge cases)
// ===========================================================================

#[test]
fn symbol_name_on_empty_language() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_name(0), None);
    assert_eq!(lang.symbol_count, 0);
}

#[test]
fn field_name_on_empty_language() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert_eq!(lang.field_name(0), None);
}

#[test]
fn is_terminal_on_empty_language() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert!(!lang.is_terminal(0));
}

#[test]
fn symbol_for_name_on_empty_language() {
    let lang = Language::builder()
        .parse_table(leak_table())
        .symbol_metadata(vec![])
        .build()
        .unwrap();
    assert_eq!(lang.symbol_for_name("anything", true), None);
}

// ===========================================================================
// 9. Multiple symbol queries – iterate and cross-check
// ===========================================================================

#[test]
fn all_symbol_names_roundtrip() {
    let lang = sample_language();
    let expected = [
        "end",
        "number",
        "+",
        "expression",
        "_statement",
        "declaration",
    ];
    for i in 0..expected.len() {
        assert_eq!(
            lang.symbol_name(i as u16),
            Some(expected[i]),
            "mismatch at index {}",
            i,
        );
    }
}

#[test]
fn all_field_names_roundtrip() {
    let lang = sample_language();
    let expected = ["left", "operator", "right"];
    for i in 0..expected.len() {
        assert_eq!(
            lang.field_name(i as u16),
            Some(expected[i]),
            "mismatch at field index {}",
            i,
        );
    }
}

#[test]
fn terminal_nonterminal_partition() {
    let lang = sample_language();
    let mut terminal_count = 0;
    let mut nonterminal_count = 0;
    for i in 0..lang.symbol_count as u16 {
        if lang.is_terminal(i) {
            terminal_count += 1;
        } else {
            nonterminal_count += 1;
        }
    }
    assert_eq!(terminal_count, 3);
    assert_eq!(nonterminal_count, 3);
}

#[test]
fn visible_hidden_partition() {
    let lang = sample_language();
    let mut visible_count = 0;
    let mut hidden_count = 0;
    for i in 0..lang.symbol_count as u16 {
        if lang.is_visible(i) {
            visible_count += 1;
        } else {
            hidden_count += 1;
        }
    }
    assert_eq!(visible_count, 3);
    assert_eq!(hidden_count, 3);
}
