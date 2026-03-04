//! Integration tests for external scanner modules.
//!
//! Covers edge cases and boundary conditions for both
//! `external_scanner` (v1) and `external_scanner_v2`.

mod test_helpers;

use adze_ir::{ExternalToken, Grammar, SymbolId};
use adze_tablegen::external_scanner::ExternalScannerGenerator as V1Generator;
use adze_tablegen::external_scanner_v2::ExternalScannerGenerator as V2Generator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a grammar pre-loaded with `n` external tokens named TOKEN_0 … TOKEN_{n-1}
/// with symbol IDs starting at `base`.
fn grammar_with_externals(n: usize, base: u16) -> Grammar {
    let mut g = Grammar::new("test".to_string());
    for i in 0..n {
        g.externals.push(ExternalToken {
            name: format!("TOKEN_{i}"),
            symbol_id: SymbolId(base + i as u16),
        });
    }
    g
}

// ===== V1 ExternalScannerGenerator tests ==================================

/// Edge case: zero states requested for the bitmap.
#[test]
fn v1_state_bitmap_zero_states() {
    let grammar = grammar_with_externals(3, 50);
    let scanner = V1Generator::new(grammar);
    let bitmap = scanner.generate_state_bitmap(0);
    assert!(
        bitmap.is_empty(),
        "zero states should produce an empty bitmap"
    );
}

/// Boundary: a single external token with a single state.
#[test]
fn v1_single_token_single_state() {
    let grammar = grammar_with_externals(1, 0);
    let scanner = V1Generator::new(grammar);

    assert_eq!(scanner.external_token_count(), 1);
    assert!(scanner.has_external_tokens());

    let bitmap = scanner.generate_state_bitmap(1);
    assert_eq!(bitmap, vec![vec![true]]);

    let symbol_map = scanner.generate_symbol_map();
    assert_eq!(symbol_map, vec![0]);
}

/// Large token count: verify bitmap dimensions stay consistent.
#[test]
fn v1_large_token_count() {
    let n = 64;
    let grammar = grammar_with_externals(n, 1000);
    let scanner = V1Generator::new(grammar);

    assert_eq!(scanner.external_token_count(), n);

    let states = 128;
    let bitmap = scanner.generate_state_bitmap(states);
    assert_eq!(bitmap.len(), states);
    for row in &bitmap {
        assert_eq!(row.len(), n);
    }
}

/// Symbol map preserves insertion order and exact IDs for non-contiguous IDs.
#[test]
fn v1_symbol_map_noncontiguous_ids() {
    let mut grammar = Grammar::new("test".to_string());
    let ids: Vec<u16> = vec![7, 42, 999, 1];
    for (i, &id) in ids.iter().enumerate() {
        grammar.externals.push(ExternalToken {
            name: format!("EXT_{i}"),
            symbol_id: SymbolId(id),
        });
    }
    let scanner = V1Generator::new(grammar);
    assert_eq!(scanner.generate_symbol_map(), ids);
}

/// Scanner interface for an empty grammar produces an empty token stream.
#[test]
fn v1_empty_interface_is_truly_empty() {
    let grammar = Grammar::new("empty".to_string());
    let scanner = V1Generator::new(grammar);

    assert!(!scanner.has_external_tokens());
    assert_eq!(scanner.external_token_count(), 0);
    assert!(scanner.generate_symbol_map().is_empty());

    let iface = scanner.generate_scanner_interface();
    assert!(
        iface.is_empty(),
        "empty grammar must not emit scanner interface code"
    );
}

// ===== V2 ExternalScannerGenerator tests ==================================

/// V2 with no external tokens: validity and symbol map should be empty / trivial.
#[test]
fn v2_empty_external_tokens() {
    let grammar = Grammar::new("test".to_string());
    let pt = test_helpers::create_minimal_parse_table(grammar.clone());
    let scanner = V2Generator::new(grammar, pt);

    assert!(!scanner.has_external_tokens());
    assert_eq!(scanner.external_token_count(), 0);
    assert!(scanner.generate_symbol_map().is_empty());

    let iface = scanner.generate_scanner_interface();
    assert!(
        iface.is_empty(),
        "empty grammar must not emit scanner interface code"
    );
}

/// V2 state validity mirrors what the parse table provides.
#[test]
fn v2_state_validity_matches_parse_table() {
    let grammar = grammar_with_externals(3, 10);
    let mut pt = test_helpers::create_minimal_parse_table(grammar.clone());

    // Hand-craft a validity matrix with mixed true/false.
    pt.external_scanner_states = vec![
        vec![true, false, true],
        vec![false, false, false],
        vec![true, true, true],
    ];

    let scanner = V2Generator::new(grammar, pt.clone());
    let validity = scanner.compute_state_validity();
    assert_eq!(validity, pt.external_scanner_states);

    // generate_state_bitmap should return the same data.
    assert_eq!(scanner.generate_state_bitmap(), validity);
}

/// V2 symbol map with duplicate symbol IDs (degenerate but valid grammar).
#[test]
fn v2_symbol_map_with_duplicate_symbol_ids() {
    let mut grammar = Grammar::new("dup".to_string());
    // Two different tokens sharing the same symbol ID – unusual but the API
    // must not panic.
    grammar.externals.push(ExternalToken {
        name: "A".to_string(),
        symbol_id: SymbolId(5),
    });
    grammar.externals.push(ExternalToken {
        name: "B".to_string(),
        symbol_id: SymbolId(5),
    });

    let pt = test_helpers::create_minimal_parse_table(grammar.clone());
    let scanner = V2Generator::new(grammar, pt);

    // Both entries map to 5.
    assert_eq!(scanner.generate_symbol_map(), vec![5, 5]);
    assert_eq!(scanner.external_token_count(), 2);
}

/// V2 scanner interface includes expected constants and data for a multi-token grammar.
#[test]
fn v2_interface_contains_constants_and_data() {
    let grammar = grammar_with_externals(4, 300);
    let mut pt = test_helpers::create_minimal_parse_table(grammar.clone());
    pt.external_scanner_states = vec![vec![true; 4]; 2];

    let scanner = V2Generator::new(grammar, pt);
    let code = scanner.generate_scanner_interface().to_string();

    assert!(
        code.contains("EXTERNAL_TOKEN_COUNT"),
        "must define EXTERNAL_TOKEN_COUNT"
    );
    assert!(code.contains("STATE_COUNT"), "must define STATE_COUNT");
    assert!(
        code.contains("EXTERNAL_SCANNER_STATES"),
        "must define EXTERNAL_SCANNER_STATES"
    );
    assert!(
        code.contains("EXTERNAL_SCANNER_SYMBOL_MAP"),
        "must define EXTERNAL_SCANNER_SYMBOL_MAP"
    );
    assert!(
        code.contains("get_valid_external_tokens"),
        "must define helper function"
    );
}

/// V2 with an empty external_scanner_states in the parse table.
#[test]
fn v2_empty_parse_table_scanner_states() {
    let grammar = grammar_with_externals(2, 10);
    let mut pt = test_helpers::create_minimal_parse_table(grammar.clone());
    // Explicitly empty – no states computed yet.
    pt.external_scanner_states = vec![];

    let scanner = V2Generator::new(grammar, pt);
    let validity = scanner.compute_state_validity();
    assert!(
        validity.is_empty(),
        "should propagate empty scanner states from parse table"
    );
}
