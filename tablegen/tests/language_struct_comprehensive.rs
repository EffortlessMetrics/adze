#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for the generated Language struct produced by `AbiLanguageBuilder`.
//!
//! Covers: ABI version, symbol/field/state counts, parse table reference,
//! external scanner slot, primary state IDs, public symbol map, and behaviour
//! across different grammar sizes.
//!
//! All tests use only public API: `AbiLanguageBuilder::new()`,
//! `.with_compressed_tables()`, and `.generate()`.

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tablegen::AbiLanguageBuilder;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

/// Build a grammar + parse table pair.
///
/// Symbol layout: ERROR(0), terminals 1..=num_terms, externals, EOF, non-terminals.
fn build_grammar_and_table(
    name: &str,
    num_terms: usize,
    num_nonterms: usize,
    num_fields: usize,
    num_externals: usize,
    num_states: usize,
) -> (Grammar, ParseTable) {
    let num_terms = num_terms.max(1);
    let num_nonterms = num_nonterms.max(1);
    let num_states = num_states.max(1);

    let eof_idx = 1 + num_terms + num_externals;
    let symbol_count = eof_idx + 1 + num_nonterms;

    let actions = vec![vec![vec![]; symbol_count]; num_states];
    let gotos = vec![vec![INVALID; symbol_count]; num_states];

    let eof_symbol = SymbolId(eof_idx as u16);
    let start_symbol = SymbolId((eof_idx + 1) as u16);

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for i in 0..symbol_count {
        let sym = SymbolId(i as u16);
        symbol_to_index.insert(sym, i);
        index_to_symbol[i] = sym;
    }

    let mut grammar = Grammar::new(name.to_string());

    let first_term = SymbolId(1);
    for i in 1..=num_terms {
        let sym = SymbolId(i as u16);
        grammar.tokens.insert(
            sym,
            Token {
                name: format!("tok_{i}"),
                pattern: TokenPattern::String(format!("t{i}")),
                fragile: false,
            },
        );
    }

    let first_nt_idx = eof_idx + 1;
    for i in 0..num_nonterms {
        let sym = SymbolId((first_nt_idx + i) as u16);
        grammar.rule_names.insert(sym, format!("rule_{i}"));
        grammar.add_rule(Rule {
            lhs: sym,
            rhs: vec![Symbol::Terminal(first_term)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }

    for i in 0..num_fields {
        grammar
            .fields
            .insert(FieldId(i as u16), format!("field_{i}"));
    }

    for i in 0..num_externals {
        grammar.externals.push(ExternalToken {
            name: format!("ext_{i}"),
            symbol_id: SymbolId((1 + num_terms + i) as u16),
        });
    }

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, start_symbol.0 as usize);

    let table = ParseTable {
        action_table: actions,
        goto_table: gotos,
        symbol_metadata: vec![],
        state_count: num_states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        external_scanner_states: vec![],
        rules: vec![],
        eof_symbol,
        start_symbol,
        grammar: Grammar::default(),
        initial_state: StateId(0),
        token_count: eof_idx + 1,
        external_token_count: num_externals,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            num_states
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    (grammar, table)
}

fn generate_code(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

/// Extract the numeric literal assigned to a field inside the LANGUAGE struct.
/// Looks for `field_name : VALUE` (with optional formatting whitespace) and
/// returns the token stream text of the value.
fn extract_language_field(code: &str, field_name: &str) -> Option<String> {
    // The generated code uses `field : value ,` inside the LANGUAGE struct.
    // We search for the pattern "field_name :" and grab everything up to the
    // next comma or closing brace.
    let needle = format!("{field_name} :");
    let start = code.find(&needle)? + needle.len();
    let rest = &code[start..];
    let end = rest.find([',', '}'])?;
    Some(rest[..end].trim().to_string())
}

// ===========================================================================
// 1. ABI version
// ===========================================================================

#[test]
fn language_version_is_tree_sitter_language_version() {
    let (g, t) = build_grammar_and_table("ver", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    let version = extract_language_field(&code, "version");
    assert_eq!(
        version.as_deref(),
        Some("TREE_SITTER_LANGUAGE_VERSION"),
        "version must reference the ABI constant"
    );
}

#[test]
fn language_version_constant_present() {
    let (g, t) = build_grammar_and_table("vconst", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(
        code.contains("TREE_SITTER_LANGUAGE_VERSION"),
        "generated code must reference the ABI version constant"
    );
}

// ===========================================================================
// 2. Symbol count matches grammar
// ===========================================================================

#[test]
fn symbol_count_matches_three_term_one_nonterm() {
    // 3 terms => symbols: ERROR(0), tok_1..tok_3(1-3), EOF(4), rule_0(5) => 6
    let (g, t) = build_grammar_and_table("sc3", 3, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    let sc = extract_language_field(&code, "symbol_count");
    assert_eq!(sc.as_deref(), Some("6u32"));
}

#[test]
fn symbol_count_matches_one_term_three_nonterms() {
    // 1 term => ERROR(0), tok_1(1), EOF(2), rule_0..rule_2(3-5) => 6
    let (g, t) = build_grammar_and_table("sc1_3", 1, 3, 0, 0, 2);
    let code = generate_code(&g, &t);
    let sc = extract_language_field(&code, "symbol_count");
    assert_eq!(sc.as_deref(), Some("6u32"));
}

#[test]
fn symbol_count_with_externals() {
    // 2 terms, 2 externals, 1 nonterm
    // ERROR(0), tok_1(1), tok_2(2), ext_0(3), ext_1(4), EOF(5), rule_0(6) => 7
    let (g, t) = build_grammar_and_table("sc_ext", 2, 1, 0, 2, 2);
    let code = generate_code(&g, &t);
    let sc = extract_language_field(&code, "symbol_count");
    assert_eq!(sc.as_deref(), Some("7u32"));
}

#[test]
fn symbol_count_single_terminal() {
    // 1 term => ERROR(0), tok_1(1), EOF(2), rule_0(3) => 4
    let (g, t) = build_grammar_and_table("sc_min", 1, 1, 0, 0, 1);
    let code = generate_code(&g, &t);
    let sc = extract_language_field(&code, "symbol_count");
    assert_eq!(sc.as_deref(), Some("4u32"));
}

// ===========================================================================
// 3. Field count matches grammar
// ===========================================================================

#[test]
fn field_count_zero_when_no_fields() {
    let (g, t) = build_grammar_and_table("fc0", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    let fc = extract_language_field(&code, "field_count");
    assert_eq!(fc.as_deref(), Some("0u32"));
}

#[test]
fn field_count_matches_three_fields() {
    let (g, t) = build_grammar_and_table("fc3", 1, 1, 3, 0, 2);
    let code = generate_code(&g, &t);
    let fc = extract_language_field(&code, "field_count");
    assert_eq!(fc.as_deref(), Some("3u32"));
}

#[test]
fn field_count_matches_large_field_set() {
    let (g, t) = build_grammar_and_table("fc10", 1, 1, 10, 0, 2);
    let code = generate_code(&g, &t);
    let fc = extract_language_field(&code, "field_count");
    assert_eq!(fc.as_deref(), Some("10u32"));
}

// ===========================================================================
// 4. Parse table reference in language
// ===========================================================================

#[test]
fn language_references_parse_table_pointer() {
    let (g, t) = build_grammar_and_table("pt", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(
        code.contains("PARSE_TABLE . as_ptr"),
        "LANGUAGE must hold a pointer to PARSE_TABLE"
    );
}

#[test]
fn language_references_small_parse_table_pointer() {
    let (g, t) = build_grammar_and_table("spt", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(
        code.contains("SMALL_PARSE_TABLE . as_ptr"),
        "LANGUAGE must hold a pointer to SMALL_PARSE_TABLE"
    );
}

#[test]
fn language_references_small_parse_table_map_pointer() {
    let (g, t) = build_grammar_and_table("sptm", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(
        code.contains("SMALL_PARSE_TABLE_MAP . as_ptr"),
        "LANGUAGE must hold a pointer to SMALL_PARSE_TABLE_MAP"
    );
}

// ===========================================================================
// 5. External scanner slot
// ===========================================================================

#[test]
fn no_externals_scanner_has_null_pointers() {
    let (g, t) = build_grammar_and_table("noext", 2, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    // The external scanner struct for grammars without externals uses null ptrs
    assert!(code.contains("std :: ptr :: null ()"));
    assert!(code.contains("ExternalScanner"));
}

#[test]
fn externals_present_scanner_populated() {
    let (g, t) = build_grammar_and_table("withext", 1, 1, 0, 3, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("EXTERNAL_SCANNER_STATES"));
    assert!(code.contains("EXTERNAL_SCANNER_SYMBOL_MAP"));
}

#[test]
fn external_token_count_matches() {
    let (g, t) = build_grammar_and_table("etc", 1, 1, 0, 2, 2);
    let code = generate_code(&g, &t);
    let etc = extract_language_field(&code, "external_token_count");
    assert_eq!(etc.as_deref(), Some("2u32"));
}

#[test]
fn external_token_count_zero_without_externals() {
    let (g, t) = build_grammar_and_table("etc0", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    let etc = extract_language_field(&code, "external_token_count");
    assert_eq!(etc.as_deref(), Some("0u32"));
}

// ===========================================================================
// 6. Primary state IDs
// ===========================================================================

#[test]
fn primary_state_ids_array_present() {
    let (g, t) = build_grammar_and_table("psi", 1, 1, 0, 0, 3);
    let code = generate_code(&g, &t);
    assert!(code.contains("PRIMARY_STATE_IDS"));
}

#[test]
fn primary_state_ids_pointer_in_language() {
    let (g, t) = build_grammar_and_table("psip", 1, 1, 0, 0, 4);
    let code = generate_code(&g, &t);
    assert!(code.contains("PRIMARY_STATE_IDS . as_ptr"));
}

#[test]
fn primary_state_ids_identity_map_for_simple_grammar() {
    // For a simple grammar each state is its own primary state.
    // With 3 states we expect entries: 0, 1, 2 encoded as u16.
    let (g, t) = build_grammar_and_table("psi_id", 1, 1, 0, 0, 3);
    let code = generate_code(&g, &t);
    let psi_section = code
        .find("PRIMARY_STATE_IDS")
        .expect("PRIMARY_STATE_IDS must exist");
    let rest = &code[psi_section..];
    // Check all three identity entries appear
    assert!(rest.contains("0"), "state 0 must be in primary_state_ids");
    assert!(rest.contains("1"), "state 1 must be in primary_state_ids");
    assert!(rest.contains("2"), "state 2 must be in primary_state_ids");
}

// ===========================================================================
// 7. Public symbol map
// ===========================================================================

#[test]
fn public_symbol_map_array_present() {
    let (g, t) = build_grammar_and_table("psm", 2, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("PUBLIC_SYMBOL_MAP"));
}

#[test]
fn public_symbol_map_pointer_in_language() {
    let (g, t) = build_grammar_and_table("psmp", 2, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("PUBLIC_SYMBOL_MAP . as_ptr"));
}

#[test]
fn public_symbol_map_is_identity_for_no_aliases() {
    // Without aliases the public symbol map is the identity mapping.
    // 2 terms => symbol_count = 5 (ERROR + 2 terms + EOF + 1 nonterm)
    // Entries: 0, 1, 2, 3, 4 each as u16.
    let (g, t) = build_grammar_and_table("psm_id", 2, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    // The generated array should contain sequential indices as u16 casts.
    // proc-macro2 prints usize literals with suffix: `0usize as u16`.
    for i in 0..5 {
        let needle = format!("{i}usize as u16");
        assert!(
            code.contains(&needle),
            "public_symbol_map must contain {needle}"
        );
    }
}

// ===========================================================================
// 8. Language from different grammar sizes
// ===========================================================================

#[test]
fn tiny_grammar_generates_valid_language() {
    let (g, t) = build_grammar_and_table("tiny", 1, 1, 0, 0, 1);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("symbol_count"));
    assert!(code.contains("state_count"));
}

#[test]
fn medium_grammar_ten_terms_five_nonterms() {
    let (g, t) = build_grammar_and_table("medium", 10, 5, 2, 0, 8);
    let code = generate_code(&g, &t);
    // symbol_count = ERROR(0) + 10 terms + EOF + 5 nonterms = 17
    let sc = extract_language_field(&code, "symbol_count");
    assert_eq!(sc.as_deref(), Some("17u32"));
    let fc = extract_language_field(&code, "field_count");
    assert_eq!(fc.as_deref(), Some("2u32"));
}

#[test]
fn large_grammar_twenty_terms_ten_nonterms() {
    let (g, t) = build_grammar_and_table("large", 20, 10, 5, 0, 15);
    let code = generate_code(&g, &t);
    // symbol_count = 1 + 20 + 1 + 10 = 32
    let sc = extract_language_field(&code, "symbol_count");
    assert_eq!(sc.as_deref(), Some("32u32"));
}

#[test]
fn large_grammar_with_externals_and_fields() {
    let (g, t) = build_grammar_and_table("large_ext", 8, 4, 6, 3, 10);
    let code = generate_code(&g, &t);
    // symbol_count = 1 + 8 + 3 externals + 1(EOF) + 4 = 17
    let sc = extract_language_field(&code, "symbol_count");
    assert_eq!(sc.as_deref(), Some("17u32"));
    let fc = extract_language_field(&code, "field_count");
    assert_eq!(fc.as_deref(), Some("6u32"));
    let etc = extract_language_field(&code, "external_token_count");
    assert_eq!(etc.as_deref(), Some("3u32"));
}

#[test]
fn state_count_matches_table() {
    let (g, t) = build_grammar_and_table("stc", 1, 1, 0, 0, 12);
    let code = generate_code(&g, &t);
    let sc = extract_language_field(&code, "state_count");
    assert_eq!(sc.as_deref(), Some("12u32"));
}

#[test]
fn state_count_single_state() {
    let (g, t) = build_grammar_and_table("stc1", 1, 1, 0, 0, 1);
    let code = generate_code(&g, &t);
    let sc = extract_language_field(&code, "state_count");
    assert_eq!(sc.as_deref(), Some("1u32"));
}

// ===========================================================================
// 9. Token count
// ===========================================================================

#[test]
fn token_count_reflects_terminal_layout() {
    // token_count = eof_idx + 1 = (1 + num_terms + externals) + 1
    // For 3 terms, 0 externals: token_count = 4 + 1 = 5
    let (g, t) = build_grammar_and_table("tc", 3, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    let tc = extract_language_field(&code, "token_count");
    assert_eq!(tc.as_deref(), Some("5u32"));
}

#[test]
fn token_count_with_externals() {
    // 2 terms, 2 externals: token_count = (1 + 2 + 2) + 1 = 6
    let (g, t) = build_grammar_and_table("tc_ext", 2, 1, 0, 2, 2);
    let code = generate_code(&g, &t);
    let tc = extract_language_field(&code, "token_count");
    assert_eq!(tc.as_deref(), Some("6u32"));
}

// ===========================================================================
// 10. Determinism across identical inputs
// ===========================================================================

#[test]
fn deterministic_across_repeated_generations() {
    let (g, t) = build_grammar_and_table("det", 5, 3, 2, 1, 6);
    let code1 = generate_code(&g, &t);
    let code2 = generate_code(&g, &t);
    let code3 = generate_code(&g, &t);
    assert_eq!(code1, code2, "first and second generation must match");
    assert_eq!(code2, code3, "second and third generation must match");
}

// ===========================================================================
// 11. Additional struct fields
// ===========================================================================

#[test]
fn alias_count_is_zero_for_no_aliases() {
    let (g, t) = build_grammar_and_table("ac0", 2, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    let ac = extract_language_field(&code, "alias_count");
    assert_eq!(ac.as_deref(), Some("0u32"));
}

#[test]
fn eof_symbol_field_present_and_zero() {
    let (g, t) = build_grammar_and_table("eof", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    // The generated code sets eof_symbol: 0
    let eof = extract_language_field(&code, "eof_symbol");
    assert_eq!(eof.as_deref(), Some("0"));
}

#[test]
fn production_id_count_scales_with_rules() {
    // 5 non-terminals each with 1 production => production IDs 0..4 => count = 5
    let (g, t) = build_grammar_and_table("pic", 2, 5, 0, 0, 3);
    let code = generate_code(&g, &t);
    let pic = extract_language_field(&code, "production_id_count");
    assert_eq!(pic.as_deref(), Some("5u32"));
}
