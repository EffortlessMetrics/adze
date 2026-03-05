#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for `AbiLanguageBuilder` output validation.
//!
//! All tests use only the public API: `AbiLanguageBuilder::new()` and `.generate()`.

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

// ===========================================================================
// 1. ABI output is non-empty
// ===========================================================================

#[test]
fn ao_v9_output_is_non_empty_minimal() {
    let (g, t) = build_grammar_and_table("ao_v9_nonempty", 1, 1, 0, 0, 1);
    let code = generate_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn ao_v9_output_is_non_empty_default_table() {
    let g = Grammar::new("ao_v9_defempty".to_string());
    let t = ParseTable::default();
    let code = generate_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn ao_v9_output_is_non_empty_with_fields() {
    let (g, t) = build_grammar_and_table("ao_v9_nonemptyfld", 2, 1, 3, 0, 2);
    let code = generate_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn ao_v9_output_is_non_empty_with_externals() {
    let (g, t) = build_grammar_and_table("ao_v9_nonemptyext", 1, 1, 0, 2, 2);
    let code = generate_code(&g, &t);
    assert!(!code.is_empty());
}

// ===========================================================================
// 2. ABI output contains grammar name
// ===========================================================================

#[test]
fn ao_v9_output_contains_grammar_name_simple() {
    let (g, t) = build_grammar_and_table("ao_v9_namesimp", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(
        code.contains("ao_v9_namesimp"),
        "output must contain the grammar name"
    );
}

#[test]
fn ao_v9_output_contains_grammar_name_in_ffi_fn() {
    let (g, t) = build_grammar_and_table("ao_v9_nameffi", 2, 1, 0, 0, 3);
    let code = generate_code(&g, &t);
    assert!(code.contains("tree_sitter_ao_v9_nameffi"));
}

#[test]
fn ao_v9_output_contains_grammar_name_large() {
    let (g, t) = build_grammar_and_table("ao_v9_namelg", 10, 5, 2, 0, 8);
    let code = generate_code(&g, &t);
    assert!(code.contains("ao_v9_namelg"));
}

#[test]
fn ao_v9_two_grammars_contain_own_names() {
    let (g1, t1) = build_grammar_and_table("ao_v9_nameone", 1, 1, 0, 0, 2);
    let (g2, t2) = build_grammar_and_table("ao_v9_nametwo", 1, 1, 0, 0, 2);
    let c1 = generate_code(&g1, &t1);
    let c2 = generate_code(&g2, &t2);
    assert!(c1.contains("ao_v9_nameone"));
    assert!(c2.contains("ao_v9_nametwo"));
    assert!(!c1.contains("ao_v9_nametwo"));
    assert!(!c2.contains("ao_v9_nameone"));
}

// ===========================================================================
// 3. ABI output contains Language / extern references
// ===========================================================================

#[test]
fn ao_v9_output_contains_language_keyword() {
    let (g, t) = build_grammar_and_table("ao_v9_langkw", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    let has_lang = code.contains("Language") || code.contains("TSLanguage");
    assert!(has_lang, "output must reference Language or TSLanguage");
}

#[test]
fn ao_v9_output_contains_extern_c() {
    let (g, t) = build_grammar_and_table("ao_v9_extc", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    let has_extern = code.contains("extern") || code.contains("no_mangle");
    assert!(has_extern, "output must have extern or no_mangle");
}

#[test]
fn ao_v9_output_contains_language_static() {
    let (g, t) = build_grammar_and_table("ao_v9_langstat", 2, 1, 0, 0, 3);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"), "must define LANGUAGE static");
}

#[test]
fn ao_v9_output_contains_abi_version() {
    let (g, t) = build_grammar_and_table("ao_v9_abiver", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("TREE_SITTER_LANGUAGE_VERSION"));
}

// ===========================================================================
// 4. ABI output is valid UTF-8
// ===========================================================================

#[test]
fn ao_v9_output_is_valid_utf8_minimal() {
    let (g, t) = build_grammar_and_table("ao_v9_utf8min", 1, 1, 0, 0, 1);
    let code = generate_code(&g, &t);
    // String type in Rust guarantees UTF-8, but confirm round-trip
    let bytes = code.as_bytes();
    let from_bytes = std::str::from_utf8(bytes);
    assert!(from_bytes.is_ok(), "output must be valid UTF-8");
}

#[test]
fn ao_v9_output_is_valid_utf8_large() {
    let (g, t) = build_grammar_and_table("ao_v9_utf8lg", 10, 5, 3, 1, 15);
    let code = generate_code(&g, &t);
    let bytes = code.as_bytes();
    assert!(std::str::from_utf8(bytes).is_ok());
}

#[test]
fn ao_v9_output_is_parseable_token_stream() {
    let (g, t) = build_grammar_and_table("ao_v9_tstream", 3, 2, 1, 0, 4);
    let code = generate_code(&g, &t);
    let parsed: Result<proc_macro2::TokenStream, _> = code.parse();
    assert!(parsed.is_ok(), "generated code must be parseable");
}

// ===========================================================================
// 5. ABI output length > 0
// ===========================================================================

#[test]
fn ao_v9_output_length_positive_minimal() {
    let (g, t) = build_grammar_and_table("ao_v9_lenmin", 1, 1, 0, 0, 1);
    let code = generate_code(&g, &t);
    assert!(!code.is_empty(), "output byte length must be positive");
}

#[test]
fn ao_v9_output_length_positive_default() {
    let g = Grammar::new("ao_v9_lendef".to_string());
    let t = ParseTable::default();
    let code = generate_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn ao_v9_output_has_substantial_content() {
    let (g, t) = build_grammar_and_table("ao_v9_lensub", 3, 2, 1, 0, 5);
    let code = generate_code(&g, &t);
    assert!(
        code.len() > 100,
        "non-trivial grammar output should be >100 bytes, got {}",
        code.len()
    );
}

// ===========================================================================
// 6. Multi-token grammar → larger output
// ===========================================================================

#[test]
fn ao_v9_more_tokens_larger_output() {
    let (g1, t1) = build_grammar_and_table("ao_v9_tok1", 1, 1, 0, 0, 2);
    let (g2, t2) = build_grammar_and_table("ao_v9_tok10", 10, 1, 0, 0, 2);
    let c1 = generate_code(&g1, &t1);
    let c2 = generate_code(&g2, &t2);
    assert!(
        c2.len() > c1.len(),
        "10 tokens ({}) should produce more output than 1 token ({})",
        c2.len(),
        c1.len()
    );
}

#[test]
fn ao_v9_more_nonterms_larger_output() {
    let (g1, t1) = build_grammar_and_table("ao_v9_nt1", 1, 1, 0, 0, 2);
    let (g2, t2) = build_grammar_and_table("ao_v9_nt8", 1, 8, 0, 0, 2);
    let c1 = generate_code(&g1, &t1);
    let c2 = generate_code(&g2, &t2);
    assert!(
        c2.len() > c1.len(),
        "8 nonterms ({}) should produce more output than 1 ({})",
        c2.len(),
        c1.len()
    );
}

#[test]
fn ao_v9_more_states_larger_output() {
    let (g1, t1) = build_grammar_and_table("ao_v9_st2", 1, 1, 0, 0, 2);
    let (g2, t2) = build_grammar_and_table("ao_v9_st20", 1, 1, 0, 0, 20);
    let c1 = generate_code(&g1, &t1);
    let c2 = generate_code(&g2, &t2);
    assert!(
        c2.len() > c1.len(),
        "20 states ({}) should produce more output than 2 ({})",
        c2.len(),
        c1.len()
    );
}

#[test]
fn ao_v9_fields_increase_output_size() {
    let (g1, t1) = build_grammar_and_table("ao_v9_fld0", 2, 1, 0, 0, 3);
    let (g2, t2) = build_grammar_and_table("ao_v9_fld5", 2, 1, 5, 0, 3);
    let c1 = generate_code(&g1, &t1);
    let c2 = generate_code(&g2, &t2);
    assert!(
        c2.len() > c1.len(),
        "5 fields ({}) should produce more output than 0 ({})",
        c2.len(),
        c1.len()
    );
}

// ===========================================================================
// 7. Grammar with precedence → ABI output
// ===========================================================================

#[test]
fn ao_v9_precedence_grammar_generates() {
    let (mut g, t) = build_grammar_and_table("ao_v9_prec", 3, 2, 0, 0, 4);
    // Add a rule with precedence
    let nt = SymbolId(5);
    g.rule_names.insert(nt, "prec_rule".to_string());
    g.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
        precedence: Some(adze_ir::PrecedenceKind::Static(10)),
        associativity: Some(adze_ir::Associativity::Left),
        fields: vec![],
        production_id: ProductionId(10),
    });
    let code = generate_code(&g, &t);
    assert!(!code.is_empty());
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn ao_v9_right_assoc_grammar_generates() {
    let (mut g, t) = build_grammar_and_table("ao_v9_rassoc", 2, 1, 0, 0, 3);
    let nt = SymbolId(4);
    g.rule_names.insert(nt, "right_rule".to_string());
    g.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: Some(adze_ir::PrecedenceKind::Static(5)),
        associativity: Some(adze_ir::Associativity::Right),
        fields: vec![],
        production_id: ProductionId(5),
    });
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn ao_v9_multiple_precedence_levels() {
    let (mut g, t) = build_grammar_and_table("ao_v9_mprec", 4, 4, 0, 0, 5);
    // nonterms start at eof_idx+1 = 6, so symbols 6..9 are valid nonterms
    for level in 0..3 {
        let nt = SymbolId((6 + level) as u16);
        g.add_rule(Rule {
            lhs: nt,
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: Some(adze_ir::PrecedenceKind::Static((level + 1) as i16)),
            associativity: Some(adze_ir::Associativity::Left),
            fields: vec![],
            production_id: ProductionId((10 + level) as u16),
        });
    }
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("TS_RULES"));
}

// ===========================================================================
// 8. Grammar with inline → ABI output
// ===========================================================================

#[test]
fn ao_v9_inline_rule_generates() {
    let (mut g, t) = build_grammar_and_table("ao_v9_inline", 2, 2, 0, 0, 3);
    let inline_sym = SymbolId(5);
    g.inline_rules.push(inline_sym);
    let code = generate_code(&g, &t);
    assert!(!code.is_empty());
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn ao_v9_multiple_inlines_generate() {
    let (mut g, t) = build_grammar_and_table("ao_v9_minline", 2, 3, 0, 0, 3);
    g.inline_rules.push(SymbolId(4));
    g.inline_rules.push(SymbolId(5));
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

// ===========================================================================
// 9. Grammar with extras → ABI output
// ===========================================================================

#[test]
fn ao_v9_extras_generates() {
    let (mut g, mut t) = build_grammar_and_table("ao_v9_extras", 2, 1, 0, 0, 3);
    let ws_id = SymbolId(1);
    g.extras.push(ws_id);
    t.extras.push(ws_id);
    let code = generate_code(&g, &t);
    assert!(!code.is_empty());
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn ao_v9_multiple_extras_generate() {
    let (mut g, mut t) = build_grammar_and_table("ao_v9_mextras", 3, 1, 0, 0, 3);
    g.extras.push(SymbolId(1));
    g.extras.push(SymbolId(2));
    t.extras.push(SymbolId(1));
    t.extras.push(SymbolId(2));
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

// ===========================================================================
// 10. Grammar with externals → ABI output
// ===========================================================================

#[test]
fn ao_v9_externals_generates() {
    let (g, t) = build_grammar_and_table("ao_v9_extgen", 1, 1, 0, 2, 2);
    let code = generate_code(&g, &t);
    assert!(!code.is_empty());
    assert!(code.contains("EXTERNAL_SCANNER"));
}

#[test]
fn ao_v9_externals_produce_scanner_fields() {
    let (g, t) = build_grammar_and_table("ao_v9_extscn", 2, 1, 0, 3, 3);
    let code = generate_code(&g, &t);
    assert!(code.contains("ExternalScanner"));
    assert!(code.contains("external_token_count"));
}

#[test]
fn ao_v9_no_externals_null_scanner() {
    let (g, t) = build_grammar_and_table("ao_v9_noext", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("null"));
}

// ===========================================================================
// 11. Determinism: same grammar → same output
// ===========================================================================

#[test]
fn ao_v9_deterministic_minimal() {
    let (g, t) = build_grammar_and_table("ao_v9_det1", 1, 1, 0, 0, 2);
    let c1 = generate_code(&g, &t);
    let c2 = generate_code(&g, &t);
    assert_eq!(c1, c2, "identical inputs must produce identical output");
}

#[test]
fn ao_v9_deterministic_complex() {
    let (g, t) = build_grammar_and_table("ao_v9_det2", 8, 4, 3, 1, 12);
    let c1 = generate_code(&g, &t);
    let c2 = generate_code(&g, &t);
    assert_eq!(c1, c2);
}

#[test]
fn ao_v9_deterministic_three_runs() {
    let (g, t) = build_grammar_and_table("ao_v9_det3", 5, 3, 2, 0, 6);
    let c1 = generate_code(&g, &t);
    let c2 = generate_code(&g, &t);
    let c3 = generate_code(&g, &t);
    assert_eq!(c1, c2);
    assert_eq!(c2, c3);
}

#[test]
fn ao_v9_deterministic_byte_level() {
    let (g, t) = build_grammar_and_table("ao_v9_det4", 3, 2, 1, 0, 4);
    let c1 = generate_code(&g, &t);
    let c2 = generate_code(&g, &t);
    assert_eq!(c1.as_bytes(), c2.as_bytes());
}

// ===========================================================================
// 12. Different grammars → different output
// ===========================================================================

#[test]
fn ao_v9_different_names_different_output() {
    let (g1, t1) = build_grammar_and_table("ao_v9_diffa", 1, 1, 0, 0, 2);
    let (g2, t2) = build_grammar_and_table("ao_v9_diffb", 1, 1, 0, 0, 2);
    let c1 = generate_code(&g1, &t1);
    let c2 = generate_code(&g2, &t2);
    assert_ne!(c1, c2, "different grammar names must produce different output");
}

#[test]
fn ao_v9_different_token_counts_different_output() {
    let (g1, t1) = build_grammar_and_table("ao_v9_difftk", 2, 1, 0, 0, 2);
    let (g2, t2) = build_grammar_and_table("ao_v9_difftk", 5, 1, 0, 0, 2);
    let c1 = generate_code(&g1, &t1);
    let c2 = generate_code(&g2, &t2);
    assert_ne!(c1, c2);
}

#[test]
fn ao_v9_different_state_counts_different_output() {
    let (g1, t1) = build_grammar_and_table("ao_v9_diffst", 1, 1, 0, 0, 3);
    let (g2, t2) = build_grammar_and_table("ao_v9_diffst", 1, 1, 0, 0, 10);
    let c1 = generate_code(&g1, &t1);
    let c2 = generate_code(&g2, &t2);
    assert_ne!(c1, c2);
}

#[test]
fn ao_v9_with_vs_without_externals_different() {
    let (g1, t1) = build_grammar_and_table("ao_v9_diffex", 1, 1, 0, 0, 2);
    let (g2, t2) = build_grammar_and_table("ao_v9_diffex", 1, 1, 0, 2, 2);
    let c1 = generate_code(&g1, &t1);
    let c2 = generate_code(&g2, &t2);
    assert_ne!(c1, c2);
}

// ===========================================================================
// 13. Output contains state references
// ===========================================================================

#[test]
fn ao_v9_output_contains_state_count() {
    let (g, t) = build_grammar_and_table("ao_v9_stref", 2, 1, 0, 0, 5);
    let code = generate_code(&g, &t);
    assert!(code.contains("state_count"));
}

#[test]
fn ao_v9_output_contains_primary_state_ids() {
    let (g, t) = build_grammar_and_table("ao_v9_prst", 1, 1, 0, 0, 3);
    let code = generate_code(&g, &t);
    assert!(code.contains("PRIMARY_STATE_IDS"));
}

#[test]
fn ao_v9_output_contains_lex_modes_for_states() {
    let (g, t) = build_grammar_and_table("ao_v9_lexm", 1, 1, 0, 0, 4);
    let code = generate_code(&g, &t);
    assert!(code.contains("LEX_MODES"));
}

#[test]
fn ao_v9_output_contains_initial_state() {
    let (g, t) = build_grammar_and_table("ao_v9_initst", 1, 1, 0, 0, 3);
    let code = generate_code(&g, &t);
    assert!(
        code.contains("StateId") || code.contains("initial_state") || code.contains("state_count")
    );
}

// ===========================================================================
// 14. Output contains symbol references
// ===========================================================================

#[test]
fn ao_v9_output_contains_symbol_count() {
    let (g, t) = build_grammar_and_table("ao_v9_symcnt", 3, 2, 0, 0, 3);
    let code = generate_code(&g, &t);
    assert!(code.contains("symbol_count"));
}

#[test]
fn ao_v9_output_contains_symbol_metadata() {
    let (g, t) = build_grammar_and_table("ao_v9_symmeta", 2, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn ao_v9_output_contains_symbol_name_ptrs() {
    let (g, t) = build_grammar_and_table("ao_v9_symptr", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("SYMBOL_NAME_PTRS"));
}

#[test]
fn ao_v9_output_contains_public_symbol_map() {
    let (g, t) = build_grammar_and_table("ao_v9_pubsym", 2, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("PUBLIC_SYMBOL_MAP"));
}

#[test]
fn ao_v9_output_contains_symbol_id_maps() {
    let (g, t) = build_grammar_and_table("ao_v9_sidmap", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("SYMBOL_ID_TO_INDEX"));
    assert!(code.contains("SYMBOL_INDEX_TO_ID"));
}

// ===========================================================================
// 15. Output contains action table data
// ===========================================================================

#[test]
fn ao_v9_output_contains_parse_table() {
    let (g, t) = build_grammar_and_table("ao_v9_ptbl", 2, 1, 0, 0, 3);
    let code = generate_code(&g, &t);
    assert!(code.contains("PARSE_TABLE"));
}

#[test]
fn ao_v9_output_contains_small_parse_table() {
    let (g, t) = build_grammar_and_table("ao_v9_sptbl", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("SMALL_PARSE_TABLE"));
}

#[test]
fn ao_v9_output_contains_parse_actions() {
    let (g, t) = build_grammar_and_table("ao_v9_pact", 3, 1, 0, 0, 4);
    let code = generate_code(&g, &t);
    assert!(code.contains("PARSE_ACTIONS"));
}

#[test]
fn ao_v9_output_contains_production_maps() {
    let (g, t) = build_grammar_and_table("ao_v9_prodmap", 1, 2, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("PRODUCTION_ID_MAP"));
    assert!(code.contains("PRODUCTION_LHS_INDEX"));
}

// ===========================================================================
// 16. Various grammar sizes
// ===========================================================================

#[test]
fn ao_v9_size_one_of_everything() {
    let (g, t) = build_grammar_and_table("ao_v9_sz1", 1, 1, 1, 1, 1);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn ao_v9_size_medium() {
    let (g, t) = build_grammar_and_table("ao_v9_szmed", 5, 3, 2, 1, 8);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("symbol_count"));
    assert!(code.contains("state_count"));
}

#[test]
fn ao_v9_size_large_terminals() {
    let (g, t) = build_grammar_and_table("ao_v9_szlgt", 25, 1, 0, 0, 5);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("token_count"));
}

#[test]
fn ao_v9_size_large_nonterms() {
    let (g, t) = build_grammar_and_table("ao_v9_szlgnt", 1, 15, 0, 0, 5);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn ao_v9_size_large_states() {
    let (g, t) = build_grammar_and_table("ao_v9_szlgs", 2, 1, 0, 0, 50);
    let code = generate_code(&g, &t);
    assert!(code.contains("state_count"));
}

#[test]
fn ao_v9_size_large_fields() {
    let (g, t) = build_grammar_and_table("ao_v9_szlgf", 2, 1, 12, 0, 3);
    let code = generate_code(&g, &t);
    assert!(code.contains("field_count"));
    assert!(code.contains("FIELD_NAME_PTRS"));
}

#[test]
fn ao_v9_size_large_externals() {
    let (g, t) = build_grammar_and_table("ao_v9_szlge", 2, 1, 0, 5, 3);
    let code = generate_code(&g, &t);
    assert!(code.contains("external_token_count"));
}

#[test]
fn ao_v9_size_everything_large() {
    let (g, t) = build_grammar_and_table("ao_v9_szall", 15, 8, 5, 3, 20);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("EXTERNAL_SCANNER"));
    assert!(code.contains("FIELD_NAME_PTRS"));
}

// ===========================================================================
// 17. Complex arithmetic grammar
// ===========================================================================

#[test]
fn ao_v9_arithmetic_grammar_basic() {
    let (mut g, t) = build_grammar_and_table("ao_v9_arith", 4, 2, 0, 0, 6);
    // Rename tokens to arithmetic operators
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(2),
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(3),
        Token {
            name: "star".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(4),
        Token {
            name: "minus".to_string(),
            pattern: TokenPattern::String("-".to_string()),
            fragile: false,
        },
    );
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("tree_sitter_ao_v9_arith"));
}

#[test]
fn ao_v9_arithmetic_grammar_with_precedence() {
    let (mut g, t) = build_grammar_and_table("ao_v9_arithp", 3, 1, 0, 0, 5);
    let expr_sym = SymbolId(5);
    g.rule_names.insert(expr_sym, "expression".to_string());
    g.add_rule(Rule {
        lhs: expr_sym,
        rhs: vec![
            Symbol::NonTerminal(expr_sym),
            Symbol::Terminal(SymbolId(2)),
            Symbol::NonTerminal(expr_sym),
        ],
        precedence: Some(adze_ir::PrecedenceKind::Static(1)),
        associativity: Some(adze_ir::Associativity::Left),
        fields: vec![],
        production_id: ProductionId(20),
    });
    g.add_rule(Rule {
        lhs: expr_sym,
        rhs: vec![
            Symbol::NonTerminal(expr_sym),
            Symbol::Terminal(SymbolId(3)),
            Symbol::NonTerminal(expr_sym),
        ],
        precedence: Some(adze_ir::PrecedenceKind::Static(2)),
        associativity: Some(adze_ir::Associativity::Left),
        fields: vec![],
        production_id: ProductionId(21),
    });
    let code = generate_code(&g, &t);
    assert!(!code.is_empty());
    assert!(code.contains("TS_RULES"));
}

#[test]
fn ao_v9_arithmetic_grammar_with_fields() {
    let (mut g, t) = build_grammar_and_table("ao_v9_arithf", 3, 2, 2, 0, 5);
    g.fields
        .insert(FieldId(0), "left".to_string());
    g.fields
        .insert(FieldId(1), "right".to_string());
    let code = generate_code(&g, &t);
    assert!(code.contains("FIELD_NAME_PTRS"));
    assert!(code.contains("field_count"));
}

// ===========================================================================
// 18. Minimal grammar → minimal output
// ===========================================================================

#[test]
fn ao_v9_minimal_grammar_produces_output() {
    let (g, t) = build_grammar_and_table("ao_v9_min", 1, 1, 0, 0, 1);
    let code = generate_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn ao_v9_minimal_has_required_statics() {
    let (g, t) = build_grammar_and_table("ao_v9_minreq", 1, 1, 0, 0, 1);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("PARSE_TABLE"));
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn ao_v9_default_grammar_produces_output() {
    let g = Grammar::new("ao_v9_defgr".to_string());
    let t = ParseTable::default();
    let code = generate_code(&g, &t);
    assert!(!code.is_empty());
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn ao_v9_minimal_smaller_than_complex() {
    let (g1, t1) = build_grammar_and_table("ao_v9_mincmp", 1, 1, 0, 0, 1);
    let (g2, t2) = build_grammar_and_table("ao_v9_mincmp", 10, 5, 3, 2, 15);
    let c1 = generate_code(&g1, &t1);
    let c2 = generate_code(&g2, &t2);
    assert!(
        c1.len() < c2.len(),
        "minimal ({}) must be smaller than complex ({})",
        c1.len(),
        c2.len()
    );
}

// ===========================================================================
// 19. ABI output structure validation
// ===========================================================================

#[test]
fn ao_v9_output_has_static_declarations() {
    let (g, t) = build_grammar_and_table("ao_v9_static", 2, 1, 0, 0, 3);
    let code = generate_code(&g, &t);
    assert!(code.contains("static"));
}

#[test]
fn ao_v9_output_has_pub_static() {
    let (g, t) = build_grammar_and_table("ao_v9_pubst", 2, 1, 0, 0, 3);
    let code = generate_code(&g, &t);
    let has_pub_static = code.contains("pub static") || code.contains("pub const");
    assert!(has_pub_static, "output must have pub static or pub const");
}

#[test]
fn ao_v9_output_has_ts_rule_entries() {
    let (g, t) = build_grammar_and_table("ao_v9_tsrule", 2, 2, 0, 0, 3);
    let code = generate_code(&g, &t);
    assert!(code.contains("TS_RULES"));
    assert!(code.contains("TSRule"));
}

#[test]
fn ao_v9_output_has_field_map_arrays() {
    let (g, t) = build_grammar_and_table("ao_v9_fldmap", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("FIELD_MAP_SLICES"));
    assert!(code.contains("FIELD_MAP_ENTRIES"));
}

#[test]
fn ao_v9_output_all_count_fields_present() {
    let (g, t) = build_grammar_and_table("ao_v9_allcnt", 3, 2, 1, 1, 4);
    let code = generate_code(&g, &t);
    let required_fields = [
        "symbol_count",
        "state_count",
        "token_count",
        "field_count",
        "production_id_count",
        "alias_count",
        "large_state_count",
        "external_token_count",
    ];
    for field in &required_fields {
        assert!(code.contains(field), "missing required field: {field}");
    }
}

#[test]
fn ao_v9_output_has_eof_symbol_field() {
    let (g, t) = build_grammar_and_table("ao_v9_eofsym", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("eof_symbol"));
}

#[test]
fn ao_v9_output_has_production_count() {
    let (g, t) = build_grammar_and_table("ao_v9_prodcnt", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("production_count"));
}

#[test]
fn ao_v9_output_has_rule_count() {
    let (g, t) = build_grammar_and_table("ao_v9_rulecnt", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("rule_count"));
}

#[test]
fn ao_v9_output_has_helper_functions() {
    let (g, t) = build_grammar_and_table("ao_v9_helpfn", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("get_symbol_index"));
    assert!(code.contains("get_symbol_id"));
}

#[test]
fn ao_v9_output_has_lexer_fn() {
    let (g, t) = build_grammar_and_table("ao_v9_lexfn", 2, 1, 0, 0, 3);
    let code = generate_code(&g, &t);
    assert!(code.contains("lexer_fn"));
}

// ===========================================================================
// 20. Output scaling with grammar complexity
// ===========================================================================

#[test]
fn ao_v9_scaling_terminals_monotonic() {
    let sizes = [1, 3, 6, 12];
    let mut prev_len = 0;
    for &n in &sizes {
        let name = format!("ao_v9_scterm{n}");
        let (g, t) = build_grammar_and_table(&name, n, 1, 0, 0, 2);
        let code = generate_code(&g, &t);
        assert!(
            code.len() > prev_len,
            "output with {n} terminals ({}) must exceed previous ({})",
            code.len(),
            prev_len,
        );
        prev_len = code.len();
    }
}

#[test]
fn ao_v9_scaling_nonterms_monotonic() {
    let sizes = [1, 4, 8];
    let mut prev_len = 0;
    for &n in &sizes {
        let name = format!("ao_v9_scnt{n}");
        let (g, t) = build_grammar_and_table(&name, 1, n, 0, 0, 2);
        let code = generate_code(&g, &t);
        assert!(
            code.len() > prev_len,
            "output with {n} nonterms ({}) must exceed previous ({})",
            code.len(),
            prev_len,
        );
        prev_len = code.len();
    }
}

#[test]
fn ao_v9_scaling_states_monotonic() {
    let sizes = [2, 8, 20];
    let mut prev_len = 0;
    for &n in &sizes {
        let name = format!("ao_v9_scst{n}");
        let (g, t) = build_grammar_and_table(&name, 1, 1, 0, 0, n);
        let code = generate_code(&g, &t);
        assert!(
            code.len() > prev_len,
            "output with {n} states ({}) must exceed previous ({})",
            code.len(),
            prev_len,
        );
        prev_len = code.len();
    }
}

#[test]
fn ao_v9_scaling_combined_complexity() {
    let (g1, t1) = build_grammar_and_table("ao_v9_sclow", 2, 1, 0, 0, 2);
    let (g2, t2) = build_grammar_and_table("ao_v9_scmid", 5, 3, 1, 0, 6);
    let (g3, t3) = build_grammar_and_table("ao_v9_schi", 12, 6, 4, 2, 15);
    let c1 = generate_code(&g1, &t1);
    let c2 = generate_code(&g2, &t2);
    let c3 = generate_code(&g3, &t3);
    assert!(c1.len() < c2.len(), "low < mid");
    assert!(c2.len() < c3.len(), "mid < high");
}

// ===========================================================================
// Additional tests for 80+ coverage
// ===========================================================================

#[test]
fn ao_v9_output_contains_small_parse_table_map() {
    let (g, t) = build_grammar_and_table("ao_v9_sptblm", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("SMALL_PARSE_TABLE_MAP"));
}

#[test]
fn ao_v9_output_eof_byte_representation() {
    let (g, t) = build_grammar_and_table("ao_v9_eofbyte", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    // "end\0" = bytes [101, 110, 100, 0]
    assert!(code.contains("101u8"), "must contain 'e' byte for 'end'");
    assert!(code.contains("110u8"), "must contain 'n' byte for 'end'");
    assert!(code.contains("100u8"), "must contain 'd' byte for 'end'");
}

#[test]
fn ao_v9_output_terminal_name_bytes() {
    let (g, t) = build_grammar_and_table("ao_v9_tnbytes", 2, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    // "tok_1" starts with 't' = 116u8
    assert!(code.contains("116u8"));
}

#[test]
fn ao_v9_field_name_bytes_present() {
    let (g, t) = build_grammar_and_table("ao_v9_fnbytes", 1, 1, 2, 0, 2);
    let code = generate_code(&g, &t);
    // "field_0" starts with 'f' = 102u8
    assert!(code.contains("102u8"));
}

#[test]
fn ao_v9_no_fields_still_has_field_ptrs() {
    let (g, t) = build_grammar_and_table("ao_v9_nofld", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("FIELD_NAME_PTRS"));
}

#[test]
fn ao_v9_extras_dont_break_symbol_metadata() {
    let (mut g, mut t) = build_grammar_and_table("ao_v9_extmeta", 2, 1, 0, 0, 2);
    g.extras.push(SymbolId(1));
    t.extras.push(SymbolId(1));
    let code = generate_code(&g, &t);
    assert!(code.contains("SYMBOL_METADATA"));
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn ao_v9_inline_doesnt_break_output() {
    let (mut g, t) = build_grammar_and_table("ao_v9_inlbrk", 2, 2, 0, 0, 3);
    g.inline_rules.push(SymbolId(4));
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("PARSE_TABLE"));
}

#[test]
fn ao_v9_supertype_doesnt_break_output() {
    let (mut g, t) = build_grammar_and_table("ao_v9_super", 2, 2, 0, 0, 3);
    g.supertypes.push(SymbolId(4));
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn ao_v9_fragile_token_generates() {
    let (mut g, t) = build_grammar_and_table("ao_v9_fragile", 2, 1, 0, 0, 2);
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "fragile_tok".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: true,
        },
    );
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn ao_v9_regex_token_pattern_generates() {
    let (mut g, t) = build_grammar_and_table("ao_v9_regex", 2, 1, 0, 0, 2);
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-zA-Z_]\w*".to_string()),
            fragile: false,
        },
    );
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn ao_v9_multiple_rules_per_nonterm() {
    let (mut g, t) = build_grammar_and_table("ao_v9_mrules", 3, 1, 0, 0, 4);
    let nt = SymbolId(5);
    g.rule_names.insert(nt, "multi_rule".to_string());
    g.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(100),
    });
    g.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(101),
    });
    g.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(SymbolId(3))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(102),
    });
    let code = generate_code(&g, &t);
    assert!(code.contains("TS_RULES"));
    let ts_rule_count = code.matches("TSRule").count();
    assert!(
        ts_rule_count >= 4,
        "expected >=4 TSRule mentions, got {ts_rule_count}"
    );
}

#[test]
fn ao_v9_empty_rhs_rule_generates() {
    // num_terms=1, num_nonterms=2 → eof_idx=2, symbol_count=5, nonterms at 3,4
    let (mut g, t) = build_grammar_and_table("ao_v9_emprhs", 1, 2, 0, 0, 2);
    let nt = SymbolId(4); // second nonterminal, already in table
    g.add_rule(Rule {
        lhs: nt,
        rhs: vec![],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(50),
    });
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn ao_v9_long_rhs_rule_generates() {
    let (mut g, t) = build_grammar_and_table("ao_v9_longrhs", 5, 1, 0, 0, 3);
    let nt = SymbolId(7);
    g.rule_names.insert(nt, "long_rule".to_string());
    g.add_rule(Rule {
        lhs: nt,
        rhs: vec![
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(3)),
            Symbol::Terminal(SymbolId(4)),
            Symbol::Terminal(SymbolId(5)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(60),
    });
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("TS_RULES"));
}

#[test]
fn ao_v9_mixed_terminal_nonterminal_rhs() {
    let (mut g, t) = build_grammar_and_table("ao_v9_mixrhs", 2, 2, 0, 0, 3);
    let nt_a = SymbolId(4);
    let nt_b = SymbolId(5);
    g.rule_names.insert(nt_a, "rule_a".to_string());
    g.rule_names.insert(nt_b, "rule_b".to_string());
    g.add_rule(Rule {
        lhs: nt_a,
        rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::NonTerminal(nt_b)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(70),
    });
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn ao_v9_output_is_consistent_across_generate_calls() {
    let (g, t) = build_grammar_and_table("ao_v9_consist", 4, 2, 1, 1, 6);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let c1 = builder.generate().to_string();
    let c2 = builder.generate().to_string();
    assert_eq!(c1, c2, "same builder instance must produce same output");
}

#[test]
fn ao_v9_builder_ref_semantics() {
    let (g, t) = build_grammar_and_table("ao_v9_refsem", 2, 1, 0, 0, 3);
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code = builder.generate().to_string();
    // Grammar and table are still accessible after builder creation
    assert!(!g.tokens.is_empty());
    assert!(!code.is_empty());
}

#[test]
fn ao_v9_output_contains_tree_sitter_prefix() {
    let (g, t) = build_grammar_and_table("ao_v9_tspfx", 1, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("tree_sitter_"));
}
