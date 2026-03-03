#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for external scanner code generation in adze-tablegen.
//!
//! Covers: grammars with zero/one/many external tokens, scanner function
//! signatures in generated code, external token IDs, and scanner state size.

mod test_helpers;

use adze_ir::{ExternalToken, Grammar, SymbolId};
use adze_tablegen::external_scanner::ExternalScannerGenerator as V1Generator;
use adze_tablegen::external_scanner_v2::ExternalScannerGenerator as V2Generator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_grammar() -> Grammar {
    Grammar::new("test".to_string())
}

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
// 1. No external tokens → no scanner functions
// ===========================================================================

#[test]
fn no_externals_v1_has_no_tokens() {
    let scanner = V1Generator::new(empty_grammar());
    assert!(!scanner.has_external_tokens());
    assert_eq!(scanner.external_token_count(), 0);
    assert!(scanner.generate_symbol_map().is_empty());
}

#[test]
fn no_externals_v1_interface_is_empty() {
    let scanner = V1Generator::new(empty_grammar());
    let code = scanner.generate_scanner_interface();
    assert!(code.is_empty(), "empty grammar must emit no scanner code");
}

#[test]
fn no_externals_v2_has_no_tokens() {
    let g = empty_grammar();
    let pt = test_helpers::create_minimal_parse_table(g.clone());
    let scanner = V2Generator::new(g, pt);
    assert!(!scanner.has_external_tokens());
    assert_eq!(scanner.external_token_count(), 0);
    assert!(scanner.generate_symbol_map().is_empty());
}

// ===========================================================================
// 2. One external token → scanner function generated
// ===========================================================================

#[test]
fn one_token_v1_has_external_tokens() {
    let scanner = V1Generator::new(grammar_with_tokens(&[("HEREDOC", 100)]));
    assert!(scanner.has_external_tokens());
    assert_eq!(scanner.external_token_count(), 1);
}

#[test]
fn one_token_v1_generates_scanner_states() {
    let scanner = V1Generator::new(grammar_with_tokens(&[("HEREDOC", 100)]));
    let code = scanner.generate_scanner_interface().to_string();
    assert!(
        code.contains("EXTERNAL_SCANNER_STATES"),
        "single-token grammar must emit EXTERNAL_SCANNER_STATES"
    );
}

#[test]
fn one_token_v1_generates_symbol_map_array() {
    let scanner = V1Generator::new(grammar_with_tokens(&[("HEREDOC", 100)]));
    let code = scanner.generate_scanner_interface().to_string();
    assert!(
        code.contains("EXTERNAL_SCANNER_SYMBOL_MAP"),
        "single-token grammar must emit EXTERNAL_SCANNER_SYMBOL_MAP"
    );
}

#[test]
fn one_token_v1_generates_scanner_data() {
    let scanner = V1Generator::new(grammar_with_tokens(&[("HEREDOC", 100)]));
    let code = scanner.generate_scanner_interface().to_string();
    assert!(
        code.contains("EXTERNAL_SCANNER_DATA"),
        "single-token grammar must emit EXTERNAL_SCANNER_DATA"
    );
}

#[test]
fn one_token_v2_generates_token_count_constant() {
    let g = grammar_with_tokens(&[("HEREDOC", 100)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true]];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(
        code.contains("EXTERNAL_TOKEN_COUNT"),
        "single-token grammar must define EXTERNAL_TOKEN_COUNT"
    );
    assert!(code.contains("1usize"), "EXTERNAL_TOKEN_COUNT should be 1");
}

#[test]
fn one_token_v2_generates_state_count_constant() {
    let g = grammar_with_tokens(&[("HEREDOC", 100)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true]; 3];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("STATE_COUNT"));
    assert!(code.contains("3usize"), "STATE_COUNT should be 3");
}

#[test]
fn one_token_v2_generates_helper_function() {
    let g = grammar_with_tokens(&[("TOK", 42)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true]];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(
        code.contains("get_valid_external_tokens"),
        "must generate helper function"
    );
}

// ===========================================================================
// 3. Multiple external tokens
// ===========================================================================

#[test]
fn multi_token_v1_counts_match() {
    let scanner = V1Generator::new(grammar_with_tokens(&[
        ("INDENT", 10),
        ("DEDENT", 11),
        ("NEWLINE", 12),
    ]));
    assert_eq!(scanner.external_token_count(), 3);
}

#[test]
fn multi_token_v1_symbol_map_length() {
    let scanner = V1Generator::new(grammar_with_tokens(&[
        ("A", 1),
        ("B", 2),
        ("C", 3),
        ("D", 4),
        ("E", 5),
    ]));
    assert_eq!(scanner.generate_symbol_map().len(), 5);
}

#[test]
fn multi_token_v2_external_token_count_in_code() {
    let g = grammar_with_tokens(&[("A", 1), ("B", 2), ("C", 3), ("D", 4)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true; 4]];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("4usize"), "EXTERNAL_TOKEN_COUNT should be 4");
}

#[test]
fn multi_token_v2_all_ids_in_symbol_map() {
    let g = grammar_with_tokens(&[("A", 50), ("B", 60), ("C", 70)]);
    let pt = test_helpers::create_minimal_parse_table(g.clone());
    let scanner = V2Generator::new(g, pt);
    assert_eq!(scanner.generate_symbol_map(), vec![50, 60, 70]);
}

#[test]
fn multi_token_v1_bitmap_rows_equal_state_count() {
    let scanner = V1Generator::new(grammar_with_tokens(&[("A", 1), ("B", 2)]));
    let bitmap = scanner.generate_state_bitmap(10);
    assert_eq!(bitmap.len(), 10);
    for row in &bitmap {
        assert_eq!(row.len(), 2);
    }
}

// ===========================================================================
// 4. Scanner function signatures
// ===========================================================================

#[test]
fn v1_signature_contains_ffi_scanner_data_type() {
    let scanner = V1Generator::new(grammar_with_tokens(&[("TOK", 1)]));
    let code = scanner.generate_scanner_interface().to_string();
    assert!(
        code.contains("TSExternalScannerData"),
        "must reference FFI scanner data type"
    );
}

#[test]
fn v1_signature_scanner_callbacks_present() {
    let scanner = V1Generator::new(grammar_with_tokens(&[("TOK", 1)]));
    let code = scanner.generate_scanner_interface().to_string();
    for callback in &["create", "destroy", "scan", "serialize", "deserialize"] {
        assert!(
            code.contains(callback),
            "generated code must reference callback: {callback}"
        );
    }
}

#[test]
fn v1_signature_callbacks_are_none() {
    let scanner = V1Generator::new(grammar_with_tokens(&[("TOK", 1)]));
    let code = scanner.generate_scanner_interface().to_string();
    // All callback fields should be None (not yet linked)
    assert!(code.contains("None"), "callbacks must be None");
}

#[test]
fn v2_signature_states_is_static_slice() {
    let g = grammar_with_tokens(&[("A", 1)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true]];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(
        code.contains("static EXTERNAL_SCANNER_STATES"),
        "states must be a static"
    );
    assert!(code.contains("& [bool]"), "states must be a bool slice");
}

#[test]
fn v2_signature_symbol_map_is_static_slice() {
    let g = grammar_with_tokens(&[("A", 1)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true]];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(
        code.contains("static EXTERNAL_SCANNER_SYMBOL_MAP"),
        "symbol map must be a static"
    );
    assert!(code.contains("& [u16]"), "symbol map must be a u16 slice");
}

#[test]
fn v2_signature_scanner_data_is_static() {
    let g = grammar_with_tokens(&[("A", 1)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true]];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(
        code.contains("static EXTERNAL_SCANNER_DATA"),
        "scanner data must be a static"
    );
}

// ===========================================================================
// 5. External token IDs in generated code
// ===========================================================================

#[test]
fn v1_generated_code_embeds_token_id() {
    let scanner = V1Generator::new(grammar_with_tokens(&[("HEREDOC", 255)]));
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("255"), "must embed token ID 255");
}

#[test]
fn v1_generated_code_embeds_all_token_ids() {
    let ids: Vec<u16> = vec![10, 200, 3000, 42];
    let tokens: Vec<(&str, u16)> = ids.iter().map(|&id| ("T", id)).collect();
    let scanner = V1Generator::new(grammar_with_tokens(&tokens));
    let code = scanner.generate_scanner_interface().to_string();
    for id in &ids {
        assert!(code.contains(&id.to_string()), "must embed token ID {id}");
    }
}

#[test]
fn v1_symbol_map_preserves_order_not_sorted() {
    let scanner = V1Generator::new(grammar_with_tokens(&[("Z", 999), ("A", 1), ("M", 500)]));
    assert_eq!(scanner.generate_symbol_map(), vec![999, 1, 500]);
}

#[test]
fn v2_generated_code_embeds_token_ids() {
    let g = grammar_with_tokens(&[("X", 77), ("Y", 88)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true, true]];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("77"), "must embed ID 77");
    assert!(code.contains("88"), "must embed ID 88");
}

#[test]
fn v2_symbol_map_boundary_ids() {
    let g = grammar_with_tokens(&[("MIN", 0), ("MAX", u16::MAX)]);
    let pt = test_helpers::create_minimal_parse_table(g.clone());
    let scanner = V2Generator::new(g, pt);
    let map = scanner.generate_symbol_map();
    assert_eq!(map[0], 0);
    assert_eq!(map[1], u16::MAX);
}

// ===========================================================================
// 6. Scanner state size in generated code
// ===========================================================================

#[test]
fn v1_bitmap_zero_states_empty() {
    let scanner = V1Generator::new(grammar_with_tokens(&[("T", 1)]));
    let bitmap = scanner.generate_state_bitmap(0);
    assert!(bitmap.is_empty());
}

#[test]
fn v1_bitmap_large_state_count() {
    let scanner = V1Generator::new(grammar_with_tokens(&[("T", 1), ("U", 2)]));
    let bitmap = scanner.generate_state_bitmap(500);
    assert_eq!(bitmap.len(), 500);
    for row in &bitmap {
        assert_eq!(row.len(), 2);
    }
}

#[test]
fn v2_state_count_matches_parse_table_states() {
    let g = grammar_with_tokens(&[("A", 1), ("B", 2)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true, false]; 7];
    let scanner = V2Generator::new(g, pt);
    let bitmap = scanner.generate_state_bitmap();
    assert_eq!(bitmap.len(), 7);
}

#[test]
fn v2_state_count_in_generated_code_matches_bitmap() {
    let g = grammar_with_tokens(&[("T", 5)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true]; 12];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("12usize"), "STATE_COUNT should be 12");
}

#[test]
fn v2_flat_bitmap_bool_count_equals_states_times_tokens() {
    let g = grammar_with_tokens(&[("A", 1), ("B", 2), ("C", 3)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    // 4 states × 3 tokens = 12 booleans
    pt.external_scanner_states = vec![
        vec![true, false, true],
        vec![false, true, false],
        vec![true, true, true],
        vec![false, false, false],
    ];
    let scanner = V2Generator::new(g, pt);
    let code = scanner.generate_scanner_interface().to_string();

    let marker = "EXTERNAL_SCANNER_STATES : & [bool] = & [";
    let start = code.find(marker).expect("must contain STATES decl") + marker.len();
    let end = code[start..].find(']').unwrap() + start;
    let content = &code[start..end];
    let true_count = content.matches("true").count();
    let false_count = content.matches("false").count();
    assert_eq!(
        true_count + false_count,
        12,
        "4 states × 3 tokens = 12 bools"
    );
}

#[test]
fn v2_empty_scanner_states_produces_zero_state_count() {
    let g = grammar_with_tokens(&[("T", 1)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![];
    let scanner = V2Generator::new(g, pt);
    let bitmap = scanner.generate_state_bitmap();
    assert!(bitmap.is_empty());
    let code = scanner.generate_scanner_interface().to_string();
    assert!(code.contains("0usize"), "STATE_COUNT should be 0");
}

#[test]
fn v2_validity_reflects_parse_table_exactly() {
    let g = grammar_with_tokens(&[("INDENT", 10), ("DEDENT", 11)]);
    let mut pt = test_helpers::create_minimal_parse_table(g.clone());
    pt.external_scanner_states = vec![vec![true, false], vec![false, true], vec![true, true]];
    let scanner = V2Generator::new(g, pt.clone());
    assert_eq!(scanner.compute_state_validity(), pt.external_scanner_states);
}

#[test]
fn v1_codegen_deterministic_across_calls() {
    let make = || {
        let g = grammar_with_tokens(&[("A", 1), ("B", 2), ("C", 3)]);
        V1Generator::new(g).generate_scanner_interface().to_string()
    };
    let a = make();
    let b = make();
    assert_eq!(a, b, "code generation must be deterministic");
}

#[test]
fn v2_codegen_deterministic_across_calls() {
    let make = || {
        let g = grammar_with_tokens(&[("X", 10), ("Y", 20)]);
        let mut pt = test_helpers::create_minimal_parse_table(g.clone());
        pt.external_scanner_states = vec![vec![true, false], vec![false, true]];
        V2Generator::new(g, pt)
            .generate_scanner_interface()
            .to_string()
    };
    let a = make();
    let b = make();
    assert_eq!(a, b, "code generation must be deterministic");
}
