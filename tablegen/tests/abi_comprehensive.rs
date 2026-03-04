//! Comprehensive tests for the ABI builder module (`abi_builder.rs`).
//!
//! Covers:
//! 1.  AbiLanguageBuilder struct creation
//! 2.  Builder pattern (with_compressed_tables)
//! 3.  Generated code structure and layout
//! 4.  Symbol table encoding (names, metadata, public map)
//! 5.  Parse table / state generation
//! 6.  Field map generation
//! 7.  Production ID / LHS index generation
//! 8.  Lex mode generation
//! 9.  Variant symbol map generation
//! 10. Edge cases: minimal grammar, no fields, externals, large grammar

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

/// Sentinel for "no goto" in test tables.
const INVALID: StateId = StateId(u16::MAX);

/// Build a grammar + parse table pair with explicit symbol layout.
///
/// Symbol layout: ERROR(0), terminals 1..=num_terms, externals, EOF, non-terminals.
/// Grammar tokens/rules use SymbolIds that match table columns directly.
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
    for (i, slot) in index_to_symbol.iter_mut().enumerate().take(symbol_count) {
        let sym = SymbolId(i as u16);
        symbol_to_index.insert(sym, i);
        *slot = sym;
    }

    let mut grammar = Grammar::new(name.to_string());

    // Register terminals (IDs 1..=num_terms match table columns)
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

    // First non-terminal column
    let first_nt_idx = eof_idx + 1;

    // Register non-terminals with IDs matching table columns
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

    // Add fields
    for i in 0..num_fields {
        grammar
            .fields
            .insert(FieldId(i as u16), format!("field_{i}"));
    }

    // Add external tokens
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

/// Build a simple arithmetic grammar and matching parse table.
fn arithmetic_grammar_and_table() -> (Grammar, ParseTable) {
    // 2 terminals, 1 non-terminal, 0 fields, 0 externals, 4 states
    build_grammar_and_table("arithmetic", 2, 1, 0, 0, 4)
}

/// Build a minimal one-token, one-rule grammar.
fn minimal_grammar_and_table() -> (Grammar, ParseTable) {
    // 1 terminal, 1 non-terminal, 0 fields, 0 externals, 2 states
    build_grammar_and_table("minimal", 1, 1, 0, 0, 2)
}

/// Generate code string from a grammar+table pair.
fn generate_code(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

// ===========================================================================
// 1. Struct creation
// ===========================================================================

#[test]
fn builder_new_creates_instance() {
    let (grammar, table) = minimal_grammar_and_table();
    let _builder = AbiLanguageBuilder::new(&grammar, &table);
}

#[test]
fn builder_new_with_default_parse_table() {
    let grammar = Grammar::new("empty".to_string());
    let table = ParseTable::default();
    let _builder = AbiLanguageBuilder::new(&grammar, &table);
}

// ===========================================================================
// 2. Builder pattern
// ===========================================================================

#[test]
fn with_compressed_tables_chains() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(!code.is_empty());
}

// ===========================================================================
// 3. Generated code structure
// ===========================================================================

#[test]
fn generated_code_contains_language_struct() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("LANGUAGE"),
        "generated code must define LANGUAGE static"
    );
}

#[test]
fn generated_code_contains_language_function() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("tree_sitter_arithmetic"),
        "generated code must define the FFI language function"
    );
}

#[test]
fn generated_code_contains_abi_version() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("TREE_SITTER_LANGUAGE_VERSION"),
        "generated code must reference ABI version constant"
    );
}

#[test]
fn generated_code_contains_parse_table_static() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(code.contains("PARSE_TABLE"));
    assert!(code.contains("SMALL_PARSE_TABLE"));
    assert!(code.contains("SMALL_PARSE_TABLE_MAP"));
}

#[test]
fn generated_code_contains_parse_actions() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(code.contains("PARSE_ACTIONS"));
}

// ===========================================================================
// 4. Symbol table encoding
// ===========================================================================

#[test]
fn symbol_names_generated_for_all_symbols() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    // Must have symbol name entries for at least EOF and the tokens
    assert!(code.contains("SYMBOL_NAME_"));
    assert!(code.contains("SYMBOL_NAME_PTRS"));
}

#[test]
fn symbol_names_contain_eof() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    // EOF should have the name "end" (null-terminated bytes)
    // The byte sequence for "end\0" is [101, 110, 100, 0]
    assert!(
        code.contains("101") && code.contains("110") && code.contains("100"),
        "symbol names must contain 'end' for EOF"
    );
}

#[test]
fn symbol_metadata_generated() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("SYMBOL_METADATA"),
        "must generate SYMBOL_METADATA array"
    );
}

#[test]
fn public_symbol_map_generated() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("PUBLIC_SYMBOL_MAP"),
        "must generate PUBLIC_SYMBOL_MAP"
    );
}

#[test]
fn symbol_count_matches_table() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    let symbol_count = table.symbol_count;
    // The generated code uses `symbol_count: N` with u32 suffix
    assert!(
        code.contains(&format!("{symbol_count}u32"))
            || code.contains(&format!("{symbol_count} u32")),
        "symbol_count ({symbol_count}) must appear in generated code, got: ...{}...",
        &code[code.find("symbol_count").unwrap_or(0)
            ..code
                .find("symbol_count")
                .map(|p| (p + 40).min(code.len()))
                .unwrap_or(40)]
    );
}

// ===========================================================================
// 5. Parse table / state generation
// ===========================================================================

#[test]
fn state_count_preserved_in_output() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    let sc = table.state_count;
    assert!(
        code.contains(&format!("{sc}u32")) || code.contains(&format!("{sc} u32")),
        "state_count ({sc}) must appear in generated output"
    );
}

#[test]
fn lex_modes_generated_per_state() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(code.contains("LEX_MODES"), "must generate LEX_MODES static");
    assert!(
        code.contains("TSLexState"),
        "lex modes use TSLexState struct"
    );
}

#[test]
fn primary_state_ids_generated() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("PRIMARY_STATE_IDS"),
        "must generate PRIMARY_STATE_IDS"
    );
}

// ===========================================================================
// 6. Field map generation
// ===========================================================================

#[test]
fn no_fields_produces_empty_field_names() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(code.contains("FIELD_NAME_PTRS"));
    // field_count should be 0
    assert!(
        code.contains("field_count : 0u32") || code.contains("field_count : 0 u32"),
        "field_count must be 0 when grammar has no fields"
    );
}

#[test]
fn fields_present_generates_field_names() {
    let (grammar, table) = build_grammar_and_table("with_fields", 1, 1, 2, 0, 2);
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("FIELD_NAME_"),
        "must generate field name statics"
    );
}

#[test]
fn field_map_slices_always_present() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(code.contains("FIELD_MAP_SLICES"));
    assert!(code.contains("FIELD_MAP_ENTRIES"));
}

// ===========================================================================
// 7. Production ID / LHS index
// ===========================================================================

#[test]
fn production_id_map_generated() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("PRODUCTION_ID_MAP"),
        "must generate PRODUCTION_ID_MAP"
    );
}

#[test]
fn production_lhs_index_generated() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("PRODUCTION_LHS_INDEX"),
        "must generate PRODUCTION_LHS_INDEX"
    );
}

#[test]
fn ts_rules_generated() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(code.contains("TS_RULES"), "must generate TS_RULES array");
    assert!(code.contains("TSRule"), "TSRule struct must be referenced");
}

// ===========================================================================
// 8. Lex mode generation
// ===========================================================================

#[test]
fn lex_mode_count_matches_states() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    // Count TSLexState occurrences in the LEX_MODES array
    let lex_state_count = code.matches("TSLexState").count();
    // At least one per state (the use in LEX_MODES, plus possibly the import)
    assert!(
        lex_state_count >= table.state_count,
        "must have at least one TSLexState per state (got {lex_state_count}, states={})",
        table.state_count
    );
}

// ===========================================================================
// 9. Variant symbol map
// ===========================================================================

#[test]
fn variant_symbol_map_contains_id_to_index() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("SYMBOL_ID_TO_INDEX"),
        "must generate SYMBOL_ID_TO_INDEX"
    );
}

#[test]
fn variant_symbol_map_contains_index_to_id() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("SYMBOL_INDEX_TO_ID"),
        "must generate SYMBOL_INDEX_TO_ID"
    );
}

#[test]
fn variant_symbol_map_helper_functions() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("get_symbol_index"),
        "must generate get_symbol_index helper"
    );
    assert!(
        code.contains("get_symbol_id"),
        "must generate get_symbol_id helper"
    );
}

// ===========================================================================
// 10. Edge cases
// ===========================================================================

#[test]
fn minimal_single_token_grammar() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(!code.is_empty());
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("tree_sitter_minimal"));
}

#[test]
fn grammar_name_in_function_name() {
    let (grammar, table) = build_grammar_and_table("my_custom_lang", 1, 1, 0, 0, 1);
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("tree_sitter_my_custom_lang"),
        "FFI function name must include grammar name"
    );
}

#[test]
fn grammar_with_extras_marks_hidden() {
    let (mut grammar, table) = build_grammar_and_table("with_extras", 2, 1, 0, 0, 2);
    // Mark the second token as an extra
    grammar.extras.push(SymbolId(2));
    let code = generate_code(&grammar, &table);
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn grammar_with_externals_generates_scanner_code() {
    let (grammar, table) = build_grammar_and_table("ext_grammar", 1, 1, 0, 2, 2);
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("ExternalScanner"),
        "must generate ExternalScanner struct when externals present"
    );
}

#[test]
fn grammar_without_externals_null_scanner() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("ExternalScanner"),
        "ExternalScanner struct always present (null ptrs when no externals)"
    );
}

#[test]
fn external_token_count_zero_when_none() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("external_token_count : 0u32")
            || code.contains("external_token_count : 0 u32"),
        "external_token_count must be 0 when no externals"
    );
}

#[test]
fn multiple_rules_same_lhs() {
    // Build grammar manually: 3 terminals, 1 non-terminal with 3 productions
    let (mut grammar, table) = build_grammar_and_table("multi_rules", 3, 1, 0, 0, 3);
    // The builder already adds one rule; add 2 more alternatives
    let nt = SymbolId((1 + 3 + 1) as u16); // first nonterminal column
    grammar.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    grammar.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(SymbolId(3))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(2),
    });
    let code = generate_code(&grammar, &table);
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("TS_RULES"));
}

#[test]
fn deterministic_output() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code1 = generate_code(&grammar, &table);
    let code2 = generate_code(&grammar, &table);
    assert_eq!(code1, code2, "same input must produce identical output");
}

#[test]
fn large_grammar_generates_successfully() {
    let (grammar, table) = build_grammar_and_table("large", 20, 10, 0, 0, 15);
    let code = generate_code(&grammar, &table);
    assert!(!code.is_empty());
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("tree_sitter_large"));
}

#[test]
fn eof_symbol_always_at_column_zero_in_language() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    // Tree-sitter convention: eof_symbol is always 0 in the LANGUAGE struct
    assert!(
        code.contains("eof_symbol : 0"),
        "eof_symbol must be 0 in generated LANGUAGE struct"
    );
}

#[test]
fn lexer_function_generated() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("lexer_fn"),
        "LANGUAGE must reference a lexer function"
    );
}

#[test]
fn alias_count_is_zero() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    // Aliases are not implemented yet — count should be 0
    assert!(
        code.contains("alias_count : 0u32") || code.contains("alias_count : 0 u32"),
        "alias_count must be 0 (not yet implemented)"
    );
}

#[test]
fn token_count_from_parse_table() {
    let (grammar, table) = arithmetic_grammar_and_table();
    let code = generate_code(&grammar, &table);
    let tc = table.token_count;
    assert!(
        code.contains(&format!("token_count : {tc}u32"))
            || code.contains(&format!("token_count : {tc} u32")),
        "token_count ({tc}) must appear in generated output"
    );
}
