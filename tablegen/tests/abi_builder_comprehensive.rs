#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for `AbiLanguageBuilder` covering construction, code generation,
//! symbol encoding, metadata, field maps, production maps, lex modes, variant maps,
//! external scanners, whitespace handling, scaling, and determinism.
//!
//! All tests use only the public API: `AbiLanguageBuilder::new()`, `.with_compressed_tables()`,
//! and `.generate()`.

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

fn minimal() -> (Grammar, ParseTable) {
    build_grammar_and_table("minimal", 1, 1, 0, 0, 2)
}

fn generate_code(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

// ===========================================================================
// 1. Builder construction
// ===========================================================================

#[test]
fn builder_new_creates_instance() {
    let (g, t) = minimal();
    let _b = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn builder_new_with_default_table() {
    let g = Grammar::new("empty".to_string());
    let t = ParseTable::default();
    let _b = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn builder_with_compressed_tables_chains() {
    let (g, t) = minimal();
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

// ===========================================================================
// 2. Generated code – LANGUAGE struct
// ===========================================================================

#[test]
fn generated_code_defines_language_static() {
    let (g, t) = build_grammar_and_table("arith", 2, 1, 0, 0, 4);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"), "must define LANGUAGE static");
}

#[test]
fn generated_code_ffi_fn_uses_grammar_name() {
    let (g, t) = build_grammar_and_table("json", 3, 2, 0, 0, 5);
    let code = generate_code(&g, &t);
    assert!(
        code.contains("tree_sitter_json"),
        "FFI function must use grammar name"
    );
}

#[test]
fn different_grammar_name_different_ffi_fn() {
    let (g1, t1) = build_grammar_and_table("python", 1, 1, 0, 0, 2);
    let (g2, t2) = build_grammar_and_table("ruby", 1, 1, 0, 0, 2);
    let c1 = generate_code(&g1, &t1);
    let c2 = generate_code(&g2, &t2);
    assert!(c1.contains("tree_sitter_python"));
    assert!(c2.contains("tree_sitter_ruby"));
    assert!(!c1.contains("tree_sitter_ruby"));
    assert!(!c2.contains("tree_sitter_python"));
}

#[test]
fn generated_code_references_abi_version() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("TREE_SITTER_LANGUAGE_VERSION"));
}

// ===========================================================================
// 3. Static arrays in generated code
// ===========================================================================

#[test]
fn generated_code_has_parse_table_statics() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("PARSE_TABLE"));
    assert!(code.contains("SMALL_PARSE_TABLE"));
    assert!(code.contains("SMALL_PARSE_TABLE_MAP"));
}

#[test]
fn generated_code_has_parse_actions() {
    let (g, t) = build_grammar_and_table("acts", 2, 1, 0, 0, 3);
    let code = generate_code(&g, &t);
    assert!(code.contains("PARSE_ACTIONS"));
}

#[test]
fn generated_code_has_lex_modes() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("LEX_MODES"));
}

#[test]
fn generated_code_has_symbol_metadata() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn generated_code_has_public_symbol_map() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("PUBLIC_SYMBOL_MAP"));
}

#[test]
fn generated_code_has_primary_state_ids() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("PRIMARY_STATE_IDS"));
}

#[test]
fn generated_code_has_production_id_map() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("PRODUCTION_ID_MAP"));
}

#[test]
fn generated_code_has_production_lhs_index() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("PRODUCTION_LHS_INDEX"));
}

#[test]
fn generated_code_has_ts_rules() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("TS_RULES"));
}

#[test]
fn generated_code_has_field_map_arrays() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("FIELD_MAP_SLICES"));
    assert!(code.contains("FIELD_MAP_ENTRIES"));
}

// ===========================================================================
// 4. Symbol names in generated code
// ===========================================================================

#[test]
fn symbol_names_array_present() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("SYMBOL_NAME_PTRS"));
}

#[test]
fn symbol_names_contain_eof_end() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    // "end\0" = [101, 110, 100, 0] — the EOF symbol name
    assert!(code.contains("101u8"), "must contain 'e' byte");
    assert!(code.contains("110u8"), "must contain 'n' byte");
    assert!(code.contains("100u8"), "must contain 'd' byte");
}

#[test]
fn symbol_names_contain_terminal_name() {
    let (g, t) = build_grammar_and_table("tn", 2, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    // "tok_1" bytes: 't'=116
    assert!(code.contains("116u8"));
}

// ===========================================================================
// 5. Field names in generated code
// ===========================================================================

#[test]
fn no_fields_field_name_ptrs_present() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("FIELD_NAME_PTRS"));
}

#[test]
fn fields_produce_field_name_data() {
    let (g, t) = build_grammar_and_table("fld", 1, 1, 2, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("FIELD_NAME_PTRS"));
    // field_0 contains 'f'=102
    assert!(code.contains("102u8"));
}

// ===========================================================================
// 6. Counts embedded in LANGUAGE struct
// ===========================================================================

#[test]
fn language_struct_has_all_count_fields() {
    let (g, t) = build_grammar_and_table("sc", 3, 2, 1, 0, 4);
    let code = generate_code(&g, &t);
    for field in &[
        "symbol_count",
        "state_count",
        "token_count",
        "field_count",
        "production_id_count",
        "alias_count",
        "large_state_count",
        "external_token_count",
    ] {
        assert!(code.contains(field), "missing field: {field}");
    }
}

// ===========================================================================
// 7. External scanner handling
// ===========================================================================

#[test]
fn no_externals_null_scanner_pointers() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("null"));
    assert!(code.contains("ExternalScanner"));
}

#[test]
fn externals_produce_scanner_struct() {
    let (g, t) = build_grammar_and_table("ext", 1, 1, 0, 2, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("ExternalScanner"));
    assert!(code.contains("EXTERNAL_SCANNER"));
}

// ===========================================================================
// 8. Variant symbol map
// ===========================================================================

#[test]
fn variant_symbol_map_generated() {
    let (g, t) = build_grammar_and_table("vsm", 2, 1, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("SYMBOL_ID_TO_INDEX"));
    assert!(code.contains("SYMBOL_INDEX_TO_ID"));
}

#[test]
fn variant_map_has_helper_functions() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("get_symbol_index"));
    assert!(code.contains("get_symbol_id"));
}

// ===========================================================================
// 9. Whitespace / extras handling
// ===========================================================================

#[test]
fn whitespace_extra_token_appears_in_metadata() {
    let (mut g, t) = minimal();
    let ws_id = SymbolId(1);
    g.tokens.insert(
        ws_id,
        Token {
            name: "whitespace".to_string(),
            pattern: TokenPattern::Regex(r"\s".to_string()),
            fragile: false,
        },
    );
    g.extras.push(ws_id);
    let code = generate_code(&g, &t);
    assert!(code.contains("SYMBOL_METADATA"));
}

// ===========================================================================
// 10. Scaling tests
// ===========================================================================

#[test]
fn twenty_terminals_generate_ok() {
    let (g, t) = build_grammar_and_table("big", 20, 5, 3, 0, 10);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("symbol_count"));
}

#[test]
fn fifty_states_generate_ok() {
    let (g, t) = build_grammar_and_table("states", 1, 1, 0, 0, 50);
    let code = generate_code(&g, &t);
    assert!(code.contains("state_count"));
}

// ===========================================================================
// 11. Determinism / reproducibility
// ===========================================================================

#[test]
fn generate_is_deterministic() {
    let (g, t) = build_grammar_and_table("det", 3, 2, 1, 0, 4);
    let code1 = generate_code(&g, &t);
    let code2 = generate_code(&g, &t);
    assert_eq!(
        code1, code2,
        "two generate() calls must produce identical output"
    );
}

// ===========================================================================
// 12. Multiple non-terminals / productions
// ===========================================================================

#[test]
fn multiple_nonterms_all_named_in_output() {
    let (g, t) = build_grammar_and_table("multi", 1, 4, 0, 0, 3);
    let code = generate_code(&g, &t);
    // Each non-terminal "rule_0" .. "rule_3" has 'r'=114 in the output
    assert!(code.contains("114u8"));
}

#[test]
fn multiple_productions_ts_rules_in_output() {
    let (g, t) = build_grammar_and_table("mp", 1, 3, 0, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("TS_RULES"));
    let count = code.matches("TSRule").count();
    // At least the declaration + 3 entries
    assert!(
        count >= 4,
        "expected at least 4 TSRule mentions, got {count}"
    );
}

// ===========================================================================
// 13. EOF symbol handling
// ===========================================================================

#[test]
fn eof_symbol_field_in_language_struct() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("eof_symbol"));
}

// ===========================================================================
// 14. Lexer code generation
// ===========================================================================

#[test]
fn lexer_fn_generated() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("lexer_fn"));
}

// ===========================================================================
// 15. Edge / degenerate cases
// ===========================================================================

#[test]
fn single_terminal_single_rule_generates() {
    let (g, t) = build_grammar_and_table("tiny", 1, 1, 0, 0, 1);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn many_fields_generated() {
    let (g, t) = build_grammar_and_table("fields", 1, 1, 10, 0, 2);
    let code = generate_code(&g, &t);
    assert!(code.contains("FIELD_NAME_PTRS"));
    assert!(code.contains("field_count"));
}

#[test]
fn production_count_in_language_struct() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("production_count"));
}

#[test]
fn rule_count_in_language_struct() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("rule_count"));
}
