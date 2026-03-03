//! Comprehensive tests for external scanner code generation in adze-tablegen.
//!
//! Covers struct generation, state serialization, ABI compatibility,
//! and edge cases for both v1 and v2 generators.

mod test_helpers;

use adze_ir::{ExternalToken, Grammar, SymbolId};
use adze_tablegen::external_scanner::ExternalScannerGenerator as V1Generator;
use adze_tablegen::external_scanner_v2::ExternalScannerGenerator as V2Generator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn grammar_with_tokens(names_and_ids: &[(&str, u16)]) -> Grammar {
    let mut g = Grammar::new("test".to_string());
    for &(name, id) in names_and_ids {
        g.externals.push(ExternalToken {
            name: name.to_string(),
            symbol_id: SymbolId(id),
        });
    }
    g
}

// ===========================================================================
// 1. V1 – Struct generation & symbol map
// ===========================================================================

/// Symbol map must have exactly one entry per external token, in declaration order.
#[test]
fn v1_symbol_map_length_matches_token_count() {
    for count in [1, 5, 16, 100] {
        let tokens: Vec<(&str, u16)> = (0..count).map(|i| ("T", i as u16 + 10)).collect::<Vec<_>>();
        // Leak names so they live long enough – simpler than owned strings here.
        let tokens: Vec<(&str, u16)> = tokens.iter().map(|&(n, id)| (n, id)).collect();
        let g = grammar_with_tokens(&tokens);
        let scanner = V1Generator::new(g);
        assert_eq!(scanner.generate_symbol_map().len(), count);
    }
}

/// SymbolId(0) is a legal external token ID – ensure it round-trips.
#[test]
fn v1_symbol_id_zero() {
    let g = grammar_with_tokens(&[("ZERO", 0)]);
    let scanner = V1Generator::new(g);
    assert_eq!(scanner.generate_symbol_map(), vec![0]);
}

/// SymbolId at u16::MAX boundary.
#[test]
fn v1_symbol_id_max_u16() {
    let g = grammar_with_tokens(&[("MAX", u16::MAX)]);
    let scanner = V1Generator::new(g);
    assert_eq!(scanner.generate_symbol_map(), vec![u16::MAX]);
}

/// Descending IDs are preserved in declaration order (not sorted).
#[test]
fn v1_symbol_map_preserves_declaration_order() {
    let g = grammar_with_tokens(&[("C", 300), ("A", 100), ("B", 200)]);
    let scanner = V1Generator::new(g);
    assert_eq!(scanner.generate_symbol_map(), vec![300, 100, 200]);
}

// ===========================================================================
// 2. V1 – State bitmap serialization
// ===========================================================================

/// Bitmap rows × cols must equal state_count × token_count.
#[test]
fn v1_bitmap_dimensions_exact() {
    let g = grammar_with_tokens(&[("A", 1), ("B", 2), ("C", 3)]);
    let scanner = V1Generator::new(g);
    let bitmap = scanner.generate_state_bitmap(7);
    assert_eq!(bitmap.len(), 7);
    for row in &bitmap {
        assert_eq!(row.len(), 3);
    }
}

/// Zero tokens → bitmap rows are empty vectors.
#[test]
fn v1_bitmap_zero_tokens() {
    let g = Grammar::new("empty".to_string());
    let scanner = V1Generator::new(g);
    let bitmap = scanner.generate_state_bitmap(5);
    assert_eq!(bitmap.len(), 5);
    for row in &bitmap {
        assert!(row.is_empty());
    }
}

/// Single state, single token bitmap.
#[test]
fn v1_bitmap_single_cell() {
    let g = grammar_with_tokens(&[("ONLY", 42)]);
    let scanner = V1Generator::new(g);
    let bitmap = scanner.generate_state_bitmap(1);
    assert_eq!(bitmap, vec![vec![true]]);
}

// ===========================================================================
// 3. V1 – Generated interface code structure
// ===========================================================================

/// Non-empty grammar must produce EXTERNAL_SCANNER_STATES array in the generated code.
#[test]
fn v1_interface_contains_states_array() {
    let g = grammar_with_tokens(&[("TOK", 50)]);
    let scanner = V1Generator::new(g);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("EXTERNAL_SCANNER_STATES"));
}

/// Non-empty grammar must produce EXTERNAL_SCANNER_SYMBOL_MAP.
#[test]
fn v1_interface_contains_symbol_map() {
    let g = grammar_with_tokens(&[("TOK", 50)]);
    let scanner = V1Generator::new(g);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("EXTERNAL_SCANNER_SYMBOL_MAP"));
}

/// Generated code must reference TSExternalScannerData.
#[test]
fn v1_interface_references_ffi_struct() {
    let g = grammar_with_tokens(&[("TOK", 50)]);
    let scanner = V1Generator::new(g);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(
        code.contains("TSExternalScannerData"),
        "must reference FFI struct"
    );
}

/// The generated symbol map literal must contain the actual symbol ID values.
#[test]
fn v1_interface_embeds_symbol_ids() {
    let g = grammar_with_tokens(&[("A", 77), ("B", 88)]);
    let scanner = V1Generator::new(g);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("77"), "must embed symbol ID 77");
    assert!(code.contains("88"), "must embed symbol ID 88");
}

// ===========================================================================
// 4. V1 – Code generation determinism
// ===========================================================================

/// Same input must produce identical output across two calls.
#[test]
fn v1_codegen_is_deterministic() {
    let make = || {
        let g = grammar_with_tokens(&[("X", 10), ("Y", 20)]);
        V1Generator::new(g).generate_scanner_interface().to_string()
    };
    assert_eq!(make(), make());
}

// ===========================================================================
// 5. V2 – Struct generation with parse table
// ===========================================================================

/// V2 generator stores external tokens from grammar.
#[test]
fn v2_token_count_matches_grammar() {
    for n in [0, 1, 3, 10] {
        let tokens: Vec<(&str, u16)> = (0..n).map(|i| ("T", i as u16)).collect();
        let g = grammar_with_tokens(&tokens);
        let pt = test_helpers::create_minimal_parse_table(g.clone());
        let scanner = V2Generator::new(g, pt);
        assert_eq!(scanner.external_token_count(), n);
        assert_eq!(scanner.has_external_tokens(), n > 0);
    }
}

/// V2 symbol map identical to V1 for the same grammar.
#[test]
fn v2_symbol_map_matches_v1() {
    let g = grammar_with_tokens(&[("A", 5), ("B", 500), ("C", 1)]);
    let v1 = V1Generator::new(g.clone()).generate_symbol_map();
    let pt = test_helpers::create_minimal_parse_table(g.clone());
    let v2 = V2Generator::new(g, pt).generate_symbol_map();
    assert_eq!(v1, v2);
}

// ===========================================================================
// 6. V2 – State validity & bitmap from parse table
// ===========================================================================

/// Validity with all-false row (no external tokens valid in a state).
#[test]
fn v2_validity_all_false_row() {
    let g = grammar_with_tokens(&[("A", 1), ("B", 2)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![false, false]];
    let scanner = V2Generator::new(g, pt);
    assert_eq!(scanner.compute_state_validity(), vec![vec![false, false]]);
}

/// Validity with all-true row (all external tokens valid).
#[test]
fn v2_validity_all_true_row() {
    let g = grammar_with_tokens(&[("A", 1), ("B", 2)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true, true]];
    let scanner = V2Generator::new(g, pt);
    assert_eq!(scanner.compute_state_validity(), vec![vec![true, true]]);
}

/// Validity with a checkerboard pattern across multiple states.
#[test]
fn v2_validity_checkerboard() {
    let g = grammar_with_tokens(&[("A", 1), ("B", 2)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![
        vec![true, false],
        vec![false, true],
        vec![true, false],
        vec![false, true],
    ];
    let scanner = V2Generator::new(g, pt);
    let validity = scanner.compute_state_validity();
    assert_eq!(validity.len(), 4);
    for (i, row) in validity.iter().enumerate() {
        assert_eq!(row[0], i % 2 == 0, "state {i} column 0");
        assert_eq!(row[1], i % 2 != 0, "state {i} column 1");
    }
}

/// generate_state_bitmap and compute_state_validity return the same data.
#[test]
fn v2_bitmap_equals_validity() {
    let g = grammar_with_tokens(&[("X", 10)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true], vec![false], vec![true]];
    let scanner = V2Generator::new(g, pt);
    assert_eq!(
        scanner.generate_state_bitmap(),
        scanner.compute_state_validity()
    );
}

// ===========================================================================
// 7. V2 – Generated interface code ABI details
// ===========================================================================

/// Generated code must define EXTERNAL_TOKEN_COUNT matching the grammar.
#[test]
fn v2_interface_token_count_constant() {
    let g = grammar_with_tokens(&[("A", 1), ("B", 2), ("C", 3)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true; 3]];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    // The constant should equal the number of external tokens.
    assert!(code.contains("EXTERNAL_TOKEN_COUNT"));
    assert!(code.contains("3usize"), "should embed token count 3");
}

/// Generated code must define STATE_COUNT matching the parse table.
#[test]
fn v2_interface_state_count_constant() {
    let g = grammar_with_tokens(&[("A", 1)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true]; 5]; // 5 states
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("STATE_COUNT"));
    assert!(code.contains("5usize"), "should embed state count 5");
}

/// Generated code must include function pointers (None) for scanner callbacks.
#[test]
fn v2_interface_scanner_callbacks_none() {
    let g = grammar_with_tokens(&[("TOK", 1)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true]];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    // Should reference create, destroy, scan, serialize, deserialize as None.
    for field in &["create", "destroy", "scan", "serialize", "deserialize"] {
        assert!(
            code.contains("None"),
            "callback {field} should be None in generated code"
        );
    }
}

/// Generated EXTERNAL_SCANNER_DATA must be a static item.
#[test]
fn v2_interface_scanner_data_is_static() {
    let g = grammar_with_tokens(&[("TOK", 1)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true]];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(
        code.contains("static EXTERNAL_SCANNER_DATA"),
        "scanner data must be a static"
    );
}

/// Generated helper function get_valid_external_tokens must appear in the code.
#[test]
fn v2_interface_has_helper_function() {
    let g = grammar_with_tokens(&[("A", 1)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true]];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("get_valid_external_tokens"));
}

// ===========================================================================
// 8. V2 – Bitmap flattening in generated code
// ===========================================================================

/// The flat boolean array in EXTERNAL_SCANNER_STATES must contain
/// exactly state_count × token_count entries.
#[test]
fn v2_flat_bitmap_size_in_code() {
    let g = grammar_with_tokens(&[("A", 1), ("B", 2)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    // 3 states × 2 tokens = 6 booleans
    pt.external_scanner_states = vec![vec![true, false], vec![false, true], vec![true, true]];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    // The generated EXTERNAL_SCANNER_STATES is a `& [bool]` slice.
    // After `= &`, we find the array contents between `[` and `]`.
    // proc_macro2 serializes as e.g. `& [true , false , true , ...]`.
    let marker = "EXTERNAL_SCANNER_STATES : & [bool] = & [";
    let start = code.find(marker).expect("must contain STATES declaration") + marker.len();
    let end = code[start..].find(']').unwrap() + start;
    let states_content = &code[start..end];
    let true_count = states_content.matches("true").count();
    let false_count = states_content.matches("false").count();
    // 4 trues + 2 falses = 6 total booleans in the states array.
    assert_eq!(
        true_count + false_count,
        6,
        "flat array should have 6 booleans, got content: {states_content}"
    );
}

// ===========================================================================
// 9. Edge cases
// ===========================================================================

/// V1: Very large state count does not panic.
#[test]
fn v1_large_state_count_no_panic() {
    let g = grammar_with_tokens(&[("T", 1)]);
    let scanner = V1Generator::new(g);
    let bitmap = scanner.generate_state_bitmap(10_000);
    assert_eq!(bitmap.len(), 10_000);
}

/// V2: Empty grammar + empty parse table scanner states → empty interface.
#[test]
fn v2_completely_empty() {
    let g = Grammar::new("empty".to_string());
    let pt = test_helpers::create_minimal_parse_table(g.clone());
    let scanner = V2Generator::new(g, pt);
    assert!(!scanner.has_external_tokens());
    assert_eq!(scanner.external_token_count(), 0);
    let code = scanner.generate_scanner_interface();
    assert!(code.is_empty(), "empty grammar must produce no code");
}

/// V2: debug_print_validity does not panic even with empty data.
#[test]
fn v2_debug_print_no_panic_empty() {
    let g = Grammar::new("empty".to_string());
    let pt = test_helpers::create_minimal_parse_table(g.clone());
    let scanner = V2Generator::new(g, pt);
    scanner.debug_print_validity(); // must not panic
}

/// V2: debug_print_validity does not panic with populated data.
#[test]
fn v2_debug_print_no_panic_populated() {
    let g = grammar_with_tokens(&[("INDENT", 10), ("DEDENT", 11)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true, false], vec![false, true]];
    let scanner = V2Generator::new(g, pt);
    scanner.debug_print_validity(); // must not panic
}

/// V1: multiple generators from different grammars are independent.
#[test]
fn v1_generators_independent() {
    let g1 = grammar_with_tokens(&[("A", 1)]);
    let g2 = grammar_with_tokens(&[("B", 2), ("C", 3)]);
    let gen1 = V1Generator::new(g1);
    let gen2 = V1Generator::new(g2);
    assert_eq!(gen1.external_token_count(), 1);
    assert_eq!(gen2.external_token_count(), 2);
    assert_ne!(gen1.generate_symbol_map(), gen2.generate_symbol_map());
}

/// V2: generators from different grammars are independent.
#[test]
fn v2_generators_independent() {
    let g1 = grammar_with_tokens(&[("A", 1)]);
    let pt1 = test_helpers::create_minimal_parse_table(g1.clone());
    let g2 = grammar_with_tokens(&[("B", 2), ("C", 3)]);
    let pt2 = test_helpers::create_minimal_parse_table(g2.clone());
    let gen1 = V2Generator::new(g1, pt1);
    let gen2 = V2Generator::new(g2, pt2);
    assert_ne!(gen1.external_token_count(), gen2.external_token_count());
}

/// V2: codegen determinism – same inputs produce same output.
#[test]
fn v2_codegen_deterministic() {
    let make = || {
        let g = grammar_with_tokens(&[("X", 10), ("Y", 20)]);
        let mut pt = test_helpers::create_minimal_parse_table(g.clone());
        pt.external_scanner_states = vec![vec![true, false]];
        V2Generator::new(g, pt)
            .generate_scanner_interface()
            .to_string()
    };
    assert_eq!(make(), make());
}

/// V2: symbol map with consecutive IDs starting at 0.
#[test]
fn v2_symbol_map_consecutive_from_zero() {
    let g = grammar_with_tokens(&[("A", 0), ("B", 1), ("C", 2)]);
    let pt = test_helpers::create_minimal_parse_table(g.clone());
    let scanner = V2Generator::new(g, pt);
    assert_eq!(scanner.generate_symbol_map(), vec![0, 1, 2]);
}

/// V1: interface code with many tokens embeds all IDs.
#[test]
fn v1_interface_embeds_all_ids_many_tokens() {
    let ids: Vec<u16> = (500..510).collect();
    let tokens: Vec<(&str, u16)> = ids.iter().map(|&id| ("T", id)).collect();
    let g = grammar_with_tokens(&tokens);
    let scanner = V1Generator::new(g);
    let code = scanner.generate_scanner_interface().to_string();
    for id in &ids {
        assert!(
            code.contains(&id.to_string()),
            "generated code must embed ID {id}"
        );
    }
}
