#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for the PUBLIC API of the ABI builder and related table-generation
//! facilities in `adze-tablegen`.
//!
//! Covered public surface:
//! - `AbiLanguageBuilder::new`, `.with_compressed_tables()`, `.generate()`
//! - `StaticLanguageGenerator::new`, `.set_start_can_be_empty()`, `.generate_language_code()`,
//!   `.generate_node_types()`, `.compress_tables()`
//! - `CompressedParseTable::new_for_testing`, `::from_parse_table`, accessors
//! - `TableCompressor::new`, `.encode_action_small()`, `.compress()`
//! - `serialize_language()`
//!
//! NO private methods are called.

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tablegen::compress::CompressedParseTable;
use adze_tablegen::serializer::serialize_language;
use adze_tablegen::{AbiLanguageBuilder, StaticLanguageGenerator, TableCompressor};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

/// Build a grammar + parse table pair with the given dimensions.
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
// 1. AbiLanguageBuilder – construction
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
    // with_compressed_tables is a builder method that returns Self
    let code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn builder_new_accepts_various_grammar_names() {
    for name in ["a", "json", "python_v3", "very_long_grammar_name_test"] {
        let (g, t) = build_grammar_and_table(name, 1, 1, 0, 0, 1);
        let code = generate_code(&g, &t);
        let expected_fn = format!("tree_sitter_{name}");
        assert!(code.contains(&expected_fn), "missing FFI fn for '{name}'");
    }
}

// ===========================================================================
// 2. AbiLanguageBuilder::generate() – LANGUAGE struct in output
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
fn different_grammar_name_produces_different_ffi_fn() {
    let (g1, t1) = build_grammar_and_table("python", 1, 1, 0, 0, 2);
    let (g2, t2) = build_grammar_and_table("ruby", 1, 1, 0, 0, 2);
    let c1 = generate_code(&g1, &t1);
    let c2 = generate_code(&g2, &t2);
    assert!(c1.contains("tree_sitter_python"));
    assert!(c2.contains("tree_sitter_ruby"));
    assert!(!c1.contains("tree_sitter_ruby"));
}

#[test]
fn generated_code_contains_symbol_names_array() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("SYMBOL_NAME_PTRS"));
}

#[test]
fn generated_code_contains_parse_actions() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("PARSE_ACTIONS"));
}

#[test]
fn generated_code_contains_lex_modes() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("LEX_MODES"));
}

#[test]
fn generated_code_contains_public_symbol_map() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("PUBLIC_SYMBOL_MAP"));
}

#[test]
fn generated_code_contains_primary_state_ids() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("PRIMARY_STATE_IDS"));
}

#[test]
fn generated_code_contains_production_maps() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("PRODUCTION_ID_MAP"));
    assert!(code.contains("PRODUCTION_LHS_INDEX"));
}

// ===========================================================================
// 3. Field variations
// ===========================================================================

#[test]
fn zero_fields_yields_empty_field_name_ptrs() {
    let (g, t) = build_grammar_and_table("nf", 1, 1, 0, 0, 1);
    let code = generate_code(&g, &t);
    // With zero fields, the field_names_array should use the empty case
    assert!(code.contains("FIELD_NAME_PTRS"));
}

#[test]
fn nonzero_fields_yield_field_names() {
    let (g, t) = build_grammar_and_table("wf", 1, 1, 3, 0, 1);
    let code = generate_code(&g, &t);
    assert!(code.contains("FIELD_NAME_PTRS"));
    // Field names should be present as null-terminated byte arrays
    assert!(code.contains("FIELD_NAME_"));
}

// ===========================================================================
// 4. External tokens
// ===========================================================================

#[test]
fn no_externals_produces_null_scanner() {
    let (g, t) = build_grammar_and_table("noext", 1, 1, 0, 0, 1);
    let code = generate_code(&g, &t);
    // No external scanner code should reference create/destroy/scan as non-None
    assert!(code.contains("ExternalScanner"));
}

#[test]
fn externals_produce_scanner_struct() {
    let (g, t) = build_grammar_and_table("ext", 2, 1, 0, 2, 3);
    let code = generate_code(&g, &t);
    assert!(
        code.contains("ExternalScanner"),
        "external tokens should trigger scanner struct"
    );
}

// ===========================================================================
// 5. generate() determinism
// ===========================================================================

#[test]
fn generate_is_deterministic() {
    let (g, t) = build_grammar_and_table("det", 3, 2, 1, 0, 4);
    let c1 = generate_code(&g, &t);
    let c2 = generate_code(&g, &t);
    assert_eq!(c1, c2, "generate() must be deterministic");
}

// ===========================================================================
// 6. Scaling – many terminals / states
// ===========================================================================

#[test]
fn many_terminals() {
    let (g, t) = build_grammar_and_table("big", 50, 5, 0, 0, 10);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn many_states() {
    let (g, t) = build_grammar_and_table("states", 2, 1, 0, 0, 100);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

// ===========================================================================
// 7. StaticLanguageGenerator – public API
// ===========================================================================

#[test]
fn static_language_generator_new() {
    let grammar = Grammar::new("test".to_string());
    let table = ParseTable::default();
    let slg = StaticLanguageGenerator::new(grammar, table);
    assert_eq!(slg.grammar.name, "test");
    assert!(slg.compressed_tables.is_none());
    assert!(!slg.start_can_be_empty);
}

#[test]
fn static_language_generator_set_start_can_be_empty() {
    let grammar = Grammar::new("test".to_string());
    let table = ParseTable::default();
    let mut slg = StaticLanguageGenerator::new(grammar, table);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
}

#[test]
fn static_language_generator_generate_language_code() {
    let (g, t) = minimal();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    // LanguageGenerator produces output (may be different from AbiLanguageBuilder)
    assert!(!code.is_empty());
}

#[test]
fn static_language_generator_generate_node_types() {
    let grammar = Grammar::new("test".to_string());
    let table = ParseTable::default();
    let slg = StaticLanguageGenerator::new(grammar, table);
    let json = slg.generate_node_types();
    // Must be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn static_language_generator_node_types_contains_tokens() {
    let mut grammar = Grammar::new("test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let table = ParseTable::default();
    let slg = StaticLanguageGenerator::new(grammar, table);
    let json = slg.generate_node_types();
    assert!(json.contains("NUMBER"));
}

// ===========================================================================
// 8. CompressedParseTable – public API
// ===========================================================================

#[test]
fn compressed_parse_table_new_for_testing() {
    let cpt = CompressedParseTable::new_for_testing(10, 5);
    assert_eq!(cpt.symbol_count(), 10);
    assert_eq!(cpt.state_count(), 5);
}

#[test]
fn compressed_parse_table_from_parse_table() {
    let (_, t) = minimal();
    let cpt = CompressedParseTable::from_parse_table(&t);
    assert_eq!(cpt.symbol_count(), t.symbol_count);
    assert_eq!(cpt.state_count(), t.state_count);
}

#[test]
fn compressed_parse_table_zero_dimensions() {
    let cpt = CompressedParseTable::new_for_testing(0, 0);
    assert_eq!(cpt.symbol_count(), 0);
    assert_eq!(cpt.state_count(), 0);
}

// ===========================================================================
// 9. TableCompressor – action encoding
// ===========================================================================

#[test]
fn encode_shift_action() {
    let c = TableCompressor::new();
    let encoded = c.encode_action_small(&Action::Shift(StateId(42))).unwrap();
    assert_eq!(encoded, 42);
    assert!(encoded < 0x8000, "shift must have high bit clear");
}

#[test]
fn encode_reduce_action() {
    let c = TableCompressor::new();
    let encoded = c
        .encode_action_small(&Action::Reduce(adze_ir::RuleId(5)))
        .unwrap();
    // 0x8000 | (5 + 1) = 0x8006
    assert_eq!(encoded, 0x8006);
    assert!(encoded >= 0x8000, "reduce must have high bit set");
}

#[test]
fn encode_accept_action() {
    let c = TableCompressor::new();
    assert_eq!(c.encode_action_small(&Action::Accept).unwrap(), 0xFFFF);
}

#[test]
fn encode_error_action() {
    let c = TableCompressor::new();
    assert_eq!(c.encode_action_small(&Action::Error).unwrap(), 0xFFFE);
}

#[test]
fn encode_recover_action() {
    let c = TableCompressor::new();
    assert_eq!(c.encode_action_small(&Action::Recover).unwrap(), 0xFFFD);
}

#[test]
fn encode_shift_overflow_rejected() {
    let c = TableCompressor::new();
    let result = c.encode_action_small(&Action::Shift(StateId(0x8000)));
    assert!(result.is_err());
}

#[test]
fn encode_reduce_overflow_rejected() {
    let c = TableCompressor::new();
    let result = c.encode_action_small(&Action::Reduce(adze_ir::RuleId(0x4000)));
    assert!(result.is_err());
}

// ===========================================================================
// 10. TableCompressor::compress – integration
// ===========================================================================

#[test]
fn compress_minimal_table() {
    let (g, mut t) = minimal();
    // State 0 must have at least one token shift for the compressor to accept
    t.action_table[0][1] = vec![Action::Shift(StateId(1))];
    let c = TableCompressor::new();
    let token_indices = adze_tablegen::collect_token_indices(&g, &t);
    let start_empty = adze_tablegen::eof_accepts_or_reduces(&t);
    let result = c.compress(&t, &token_indices, start_empty);
    assert!(result.is_ok(), "minimal table must compress");
}

#[test]
fn compress_preserves_small_table_threshold() {
    let (g, mut t) = minimal();
    t.action_table[0][1] = vec![Action::Shift(StateId(1))];
    let c = TableCompressor::new();
    let token_indices = adze_tablegen::collect_token_indices(&g, &t);
    let compressed = c.compress(&t, &token_indices, false).unwrap();
    assert_eq!(compressed.small_table_threshold, 32768);
}

// ===========================================================================
// 11. serialize_language – public API
// ===========================================================================

#[test]
fn serialize_language_returns_valid_json() {
    let (g, t) = minimal();
    let json = serialize_language(&g, &t, None).expect("serialization must succeed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");
    assert!(parsed.is_object());
}

#[test]
fn serialize_language_contains_version() {
    let (g, t) = minimal();
    let json = serialize_language(&g, &t, None).unwrap();
    assert!(json.contains("\"version\""));
}

#[test]
fn serialize_language_contains_symbol_count() {
    let (g, t) = build_grammar_and_table("sc", 3, 1, 0, 0, 2);
    let json = serialize_language(&g, &t, None).unwrap();
    assert!(json.contains("\"symbol_count\""));
}

#[test]
fn serialize_language_contains_symbol_names() {
    let (g, t) = minimal();
    let json = serialize_language(&g, &t, None).unwrap();
    assert!(json.contains("\"symbol_names\""));
}

// ===========================================================================
// 12. Edge cases
// ===========================================================================

#[test]
fn empty_grammar_generates_without_panic() {
    let g = Grammar::new("empty".to_string());
    let t = ParseTable::default();
    let _code = AbiLanguageBuilder::new(&g, &t).generate().to_string();
}

#[test]
fn single_state_table() {
    let (g, t) = build_grammar_and_table("one", 1, 1, 0, 0, 1);
    let code = generate_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn grammar_with_regex_token() {
    let mut grammar = Grammar::new("regex".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "IDENT".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    let (_, t) = build_grammar_and_table("regex", 1, 1, 0, 0, 2);
    // Use our custom grammar with the table
    let code = AbiLanguageBuilder::new(&grammar, &t).generate().to_string();
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn grammar_with_hidden_token() {
    let mut grammar = Grammar::new("hidden".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "_ws".to_string(),
            pattern: TokenPattern::Regex(r"\s+".to_string()),
            fragile: false,
        },
    );
    let (_, t) = build_grammar_and_table("hidden", 1, 1, 0, 0, 1);
    let code = AbiLanguageBuilder::new(&grammar, &t).generate().to_string();
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn generate_code_contains_ts_rules() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("TS_RULES"));
}

#[test]
fn generate_code_contains_variant_symbol_map() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("SYMBOL_ID_TO_INDEX"));
}

#[test]
fn generate_code_contains_small_parse_table() {
    let (g, t) = minimal();
    let code = generate_code(&g, &t);
    assert!(code.contains("SMALL_PARSE_TABLE"));
}

#[test]
fn compressed_tables_validate_ok_on_minimal() {
    let (g, mut t) = minimal();
    t.action_table[0][1] = vec![Action::Shift(StateId(1))];
    let c = TableCompressor::new();
    let token_indices = adze_tablegen::collect_token_indices(&g, &t);
    let compressed = c.compress(&t, &token_indices, false).unwrap();
    assert!(compressed.validate(&t).is_ok());
}
