#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for ABI version handling in adze-tablegen.
//!
//! Covers:
//! - ABI version constant value (tree-sitter compat)
//! - ABI version in generated code
//! - Version field in Language struct
//! - Version compatibility checks
//! - Generated code contains correct ABI version string
//! - ABI layout correctness

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tablegen::AbiLanguageBuilder;
use adze_tablegen::LanguageBuilder;
use adze_tablegen::abi::{
    self, ExternalScanner, TREE_SITTER_LANGUAGE_VERSION,
    TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION, TSFieldId, TSLanguage, TSLexState, TSParseAction,
    TSStateId, TSSymbol, create_symbol_metadata,
};
use adze_tablegen::compress::CompressedParseTable;
use adze_tablegen::validation::{
    LanguageValidator, TSExternalScannerData, TSLanguage as ValidationTSLanguage, ValidationError,
};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

/// Build a grammar + parse table pair suitable for code generation tests.
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

fn minimal_grammar_and_table() -> (Grammar, ParseTable) {
    build_grammar_and_table("minimal", 1, 1, 0, 0, 2)
}

/// Create a test Language struct for validation tests (mirrors the crate-internal helper).
fn create_test_language() -> ValidationTSLanguage {
    ValidationTSLanguage {
        version: 15,
        symbol_count: 10,
        alias_count: 0,
        token_count: 5,
        external_token_count: 0,
        state_count: 20,
        large_state_count: 0,
        production_id_count: 0,
        field_count: 0,
        max_alias_sequence_length: 0,
        parse_table: std::ptr::null(),
        small_parse_table: std::ptr::null(),
        small_parse_table_map: std::ptr::null(),
        parse_actions: std::ptr::null(),
        symbol_names: std::ptr::null(),
        field_names: std::ptr::null(),
        field_map_slices: std::ptr::null(),
        field_map_entries: std::ptr::null(),
        symbol_metadata: std::ptr::null(),
        public_symbol_map: std::ptr::null(),
        alias_map: std::ptr::null(),
        alias_sequences: std::ptr::null(),
        lex_modes: std::ptr::null(),
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner_data: TSExternalScannerData {
            states: std::ptr::null(),
            symbol_map: std::ptr::null(),
            create: None,
            destroy: None,
            scan: None,
            serialize: None,
            deserialize: None,
        },
        primary_state_ids: std::ptr::null(),
    }
}

fn generate_code(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

// ===========================================================================
// 1. ABI version constant value
// ===========================================================================

#[test]
fn abi_version_constant_is_15() {
    assert_eq!(TREE_SITTER_LANGUAGE_VERSION, 15);
}

#[test]
fn abi_version_constant_type_is_u32() {
    let v: u32 = TREE_SITTER_LANGUAGE_VERSION;
    assert_eq!(v, 15u32);
}

#[test]
fn min_compatible_version_is_13() {
    assert_eq!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION, 13);
}

#[test]
fn version_at_least_min_compatible() {
    assert!(TREE_SITTER_LANGUAGE_VERSION >= TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION);
}

#[test]
fn version_range_is_valid() {
    let range = TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION..=TREE_SITTER_LANGUAGE_VERSION;
    assert!(range.contains(&TREE_SITTER_LANGUAGE_VERSION));
    assert!(range.contains(&TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION));
    assert!(range.contains(&14));
}

// ===========================================================================
// 2. ABI version in generated code
// ===========================================================================

#[test]
fn generated_code_references_abi_version_constant() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("TREE_SITTER_LANGUAGE_VERSION"),
        "generated code must reference TREE_SITTER_LANGUAGE_VERSION"
    );
}

#[test]
fn generated_code_uses_version_in_language_struct() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("version") && code.contains("TREE_SITTER_LANGUAGE_VERSION"),
        "LANGUAGE struct must set version field from ABI constant"
    );
}

#[test]
fn generated_code_imports_abi_version() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    // The generated code should use/import TREE_SITTER_LANGUAGE_VERSION
    assert!(
        code.contains("TREE_SITTER_LANGUAGE_VERSION"),
        "generated code must import or use TREE_SITTER_LANGUAGE_VERSION"
    );
}

#[test]
fn generated_code_version_not_hardcoded_numeric() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    // Find the version field assignment in LANGUAGE struct
    // It should use the constant, not a hardcoded number
    let has_constant = code.contains("version : TREE_SITTER_LANGUAGE_VERSION")
        || code.contains("version: TREE_SITTER_LANGUAGE_VERSION");
    assert!(
        has_constant,
        "version field should use TREE_SITTER_LANGUAGE_VERSION constant, not a hardcoded literal"
    );
}

#[test]
fn different_grammars_same_abi_version_string() {
    let (g1, t1) = build_grammar_and_table("alpha", 2, 1, 0, 0, 3);
    let (g2, t2) = build_grammar_and_table("beta", 3, 2, 1, 0, 5);
    let code1 = generate_code(&g1, &t1);
    let code2 = generate_code(&g2, &t2);
    // Both should reference the same ABI version constant
    assert!(code1.contains("TREE_SITTER_LANGUAGE_VERSION"));
    assert!(code2.contains("TREE_SITTER_LANGUAGE_VERSION"));
}

// ===========================================================================
// 3. Version field in Language struct (validation module)
// ===========================================================================

#[test]
fn language_builder_sets_version_to_15() {
    let (grammar, table) = minimal_grammar_and_table();
    let builder = LanguageBuilder::new(grammar, table);
    let language = builder
        .generate_language()
        .expect("should generate language");
    assert_eq!(language.version, 15);
}

#[test]
fn language_builder_version_matches_constant() {
    let (grammar, table) = minimal_grammar_and_table();
    let builder = LanguageBuilder::new(grammar, table);
    let language = builder
        .generate_language()
        .expect("should generate language");
    assert_eq!(language.version, TREE_SITTER_LANGUAGE_VERSION);
}

#[test]
fn language_builder_different_grammars_same_version() {
    let (g1, t1) = build_grammar_and_table("one", 1, 1, 0, 0, 2);
    let (g2, t2) = build_grammar_and_table("two", 3, 2, 1, 0, 4);
    let l1 = LanguageBuilder::new(g1, t1)
        .generate_language()
        .expect("l1");
    let l2 = LanguageBuilder::new(g2, t2)
        .generate_language()
        .expect("l2");
    assert_eq!(l1.version, l2.version);
    assert_eq!(l1.version, 15);
}

// ===========================================================================
// 4. Version compatibility checks (validator)
// ===========================================================================

#[test]
fn validator_accepts_version_15() {
    let language = create_test_language();
    assert_eq!(language.version, 15);
    let tables = CompressedParseTable::new_for_testing(
        language.symbol_count as usize,
        language.state_count as usize,
    );
    let validator = LanguageValidator::new(&language, &tables);
    let result = validator.validate();
    // Version should pass even if other things fail
    if let Err(errors) = &result {
        for err in errors {
            assert!(
                !matches!(err, ValidationError::InvalidVersion { .. }),
                "version 15 should not produce InvalidVersion error"
            );
        }
    }
}

#[test]
fn validator_rejects_version_14() {
    let mut language = create_test_language();
    language.version = 14;
    let tables = CompressedParseTable::new_for_testing(
        language.symbol_count as usize,
        language.state_count as usize,
    );
    let validator = LanguageValidator::new(&language, &tables);
    let result = validator.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::InvalidVersion {
            expected: 15,
            actual: 14
        }
    )));
}

#[test]
fn validator_rejects_version_0() {
    let mut language = create_test_language();
    language.version = 0;
    let tables = CompressedParseTable::new_for_testing(
        language.symbol_count as usize,
        language.state_count as usize,
    );
    let validator = LanguageValidator::new(&language, &tables);
    let result = validator.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::InvalidVersion {
            expected: 15,
            actual: 0
        }
    )));
}

#[test]
fn validator_rejects_version_16() {
    let mut language = create_test_language();
    language.version = 16;
    let tables = CompressedParseTable::new_for_testing(
        language.symbol_count as usize,
        language.state_count as usize,
    );
    let validator = LanguageValidator::new(&language, &tables);
    let result = validator.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::InvalidVersion {
            expected: 15,
            actual: 16
        }
    )));
}

#[test]
fn validator_rejects_version_u32_max() {
    let mut language = create_test_language();
    language.version = u32::MAX;
    let tables = CompressedParseTable::new_for_testing(
        language.symbol_count as usize,
        language.state_count as usize,
    );
    let validator = LanguageValidator::new(&language, &tables);
    let result = validator.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidVersion { expected: 15, .. }))
    );
}

#[test]
fn validation_error_invalid_version_has_correct_fields() {
    let err = ValidationError::InvalidVersion {
        expected: 15,
        actual: 12,
    };
    match err {
        ValidationError::InvalidVersion { expected, actual } => {
            assert_eq!(expected, 15);
            assert_eq!(actual, 12);
        }
        _ => panic!("wrong variant"),
    }
}

// ===========================================================================
// 5. Generated code contains correct ABI version string
// ===========================================================================

#[test]
fn generated_code_contains_language_static() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("LANGUAGE"),
        "must define LANGUAGE static in output"
    );
}

#[test]
fn generated_code_contains_tslanguage_type() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("TSLanguage"),
        "must reference TSLanguage type in output"
    );
}

#[test]
fn generated_code_version_field_appears_first_in_struct() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    // version field should appear before symbol_count in the LANGUAGE initializer
    let version_pos = code.find("version");
    let symbol_count_pos = code.find("symbol_count");
    assert!(version_pos.is_some(), "version field must be present");
    assert!(
        symbol_count_pos.is_some(),
        "symbol_count field must be present"
    );
    assert!(
        version_pos.unwrap() < symbol_count_pos.unwrap(),
        "version must come before symbol_count in LANGUAGE struct"
    );
}

#[test]
fn generated_code_ffi_function_returns_language_ptr() {
    let (grammar, table) = build_grammar_and_table("test_lang", 1, 1, 0, 0, 2);
    let code = generate_code(&grammar, &table);
    // Should have a tree_sitter_<name> function
    assert!(
        code.contains("tree_sitter_test_lang"),
        "must generate tree_sitter_<name> FFI function"
    );
}

#[test]
fn generated_code_language_struct_is_pub_static() {
    let (grammar, table) = minimal_grammar_and_table();
    let code = generate_code(&grammar, &table);
    assert!(
        code.contains("pub static LANGUAGE"),
        "LANGUAGE must be pub static"
    );
}

// ===========================================================================
// 6. ABI layout correctness
// ===========================================================================

#[test]
fn ts_symbol_size_is_2_bytes() {
    assert_eq!(std::mem::size_of::<TSSymbol>(), 2);
}

#[test]
fn ts_state_id_size_is_2_bytes() {
    assert_eq!(std::mem::size_of::<TSStateId>(), 2);
}

#[test]
fn ts_field_id_size_is_2_bytes() {
    assert_eq!(std::mem::size_of::<TSFieldId>(), 2);
}

#[test]
fn ts_parse_action_size_is_6_bytes() {
    assert_eq!(std::mem::size_of::<TSParseAction>(), 6);
}

#[test]
fn ts_lex_state_size_is_4_bytes() {
    assert_eq!(std::mem::size_of::<TSLexState>(), 4);
}

#[test]
fn ts_language_alignment_matches_pointer() {
    assert_eq!(
        std::mem::align_of::<TSLanguage>(),
        std::mem::align_of::<*const u8>()
    );
}

#[test]
fn external_scanner_default_has_null_pointers() {
    let scanner = ExternalScanner::default();
    assert!(scanner.states.is_null());
    assert!(scanner.symbol_map.is_null());
    assert!(scanner.create.is_none());
    assert!(scanner.destroy.is_none());
    assert!(scanner.scan.is_none());
    assert!(scanner.serialize.is_none());
    assert!(scanner.deserialize.is_none());
}

#[test]
fn symbol_metadata_flags_are_distinct_bits() {
    assert_eq!(abi::symbol_metadata::VISIBLE, 0x01);
    assert_eq!(abi::symbol_metadata::NAMED, 0x02);
    assert_eq!(abi::symbol_metadata::HIDDEN, 0x04);
    assert_eq!(abi::symbol_metadata::AUXILIARY, 0x08);
    assert_eq!(abi::symbol_metadata::SUPERTYPE, 0x10);
    // All flags are distinct powers of 2
    let flags = [0x01u8, 0x02, 0x04, 0x08, 0x10];
    for i in 0..flags.len() {
        for j in (i + 1)..flags.len() {
            assert_eq!(flags[i] & flags[j], 0, "flags must be non-overlapping");
        }
    }
}

#[test]
fn create_symbol_metadata_combines_flags_correctly() {
    let meta = create_symbol_metadata(true, true, false, false, false);
    assert_eq!(
        meta,
        abi::symbol_metadata::VISIBLE | abi::symbol_metadata::NAMED
    );

    let meta2 = create_symbol_metadata(false, false, true, true, true);
    assert_eq!(
        meta2,
        abi::symbol_metadata::HIDDEN
            | abi::symbol_metadata::AUXILIARY
            | abi::symbol_metadata::SUPERTYPE
    );

    let meta_all = create_symbol_metadata(true, true, true, true, true);
    assert_eq!(meta_all, 0x01 | 0x02 | 0x04 | 0x08 | 0x10);
}

#[test]
fn create_symbol_metadata_all_false_is_zero() {
    let meta = create_symbol_metadata(false, false, false, false, false);
    assert_eq!(meta, 0);
}
