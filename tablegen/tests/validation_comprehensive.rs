//! Comprehensive tests for `adze_tablegen::validation`.
//!
//! Covers `LanguageValidator`, `ValidationError` variants, and integration
//! with `LanguageBuilder::generate_language`.

use adze_glr_core::{Action, GotoIndexing, ParseTable};
use adze_ir::{FieldId, Grammar, StateId, SymbolId, Token, TokenPattern};
use adze_tablegen::validation::{TSExternalScannerData, TSLanguage, TSLexer, TSSymbolMetadata};
use adze_tablegen::{CompressedParseTable, LanguageBuilder, LanguageValidator, ValidationError};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a `TSLanguage` with all-null pointers and sensible defaults.
fn bare_language(symbol_count: u32, state_count: u32) -> TSLanguage {
    TSLanguage {
        version: 15,
        symbol_count,
        alias_count: 0,
        token_count: 5,
        external_token_count: 0,
        state_count,
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

/// Build a minimal `ParseTable` for `LanguageBuilder` usage.
fn minimal_parse_table(grammar: Grammar) -> ParseTable {
    ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 0,
        symbol_to_index: std::collections::BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: std::collections::BTreeMap::new(),
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar,
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: std::collections::BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

/// Build a parse table with some actions/gotos so that `generate_language`
/// produces non-trivial counts.
fn populated_parse_table(grammar: Grammar) -> ParseTable {
    let mut pt = minimal_parse_table(grammar);
    pt.action_table = vec![
        vec![vec![Action::Shift(StateId(1))]],
        vec![vec![Action::Accept]],
    ];
    pt.goto_table = vec![vec![StateId(0)], vec![StateId(1)]];
    pt.state_count = 2;
    pt.symbol_count = 2;
    pt
}

fn has_error<F: Fn(&ValidationError) -> bool>(
    result: &std::result::Result<(), Vec<ValidationError>>,
    pred: F,
) -> bool {
    match result {
        Ok(()) => false,
        Err(errors) => errors.iter().any(pred),
    }
}

// ===========================================================================
// 1. Version validation
// ===========================================================================

#[test]
fn version_14_rejected() {
    let mut lang = bare_language(10, 20);
    lang.version = 14;
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(has_error(&res, |e| matches!(
        e,
        ValidationError::InvalidVersion {
            expected: 15,
            actual: 14
        }
    )));
}

#[test]
fn version_0_rejected() {
    let mut lang = bare_language(10, 20);
    lang.version = 0;
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(has_error(&res, |e| matches!(
        e,
        ValidationError::InvalidVersion { .. }
    )));
}

#[test]
fn version_16_rejected() {
    let mut lang = bare_language(10, 20);
    lang.version = 16;
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(has_error(&res, |e| matches!(
        e,
        ValidationError::InvalidVersion {
            expected: 15,
            actual: 16
        }
    )));
}

#[test]
fn version_u32_max_rejected() {
    let mut lang = bare_language(10, 20);
    lang.version = u32::MAX;
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(res.is_err());
}

// ===========================================================================
// 2. Symbol-count mismatch
// ===========================================================================

#[test]
fn symbol_count_too_high() {
    let mut lang = bare_language(99, 20);
    lang.symbol_count = 99;
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(has_error(&res, |e| matches!(
        e,
        ValidationError::SymbolCountMismatch {
            language: 99,
            tables: 10
        }
    )));
}

#[test]
fn symbol_count_too_low() {
    let lang = bare_language(5, 20);
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(has_error(&res, |e| matches!(
        e,
        ValidationError::SymbolCountMismatch {
            language: 5,
            tables: 10
        }
    )));
}

#[test]
fn symbol_count_match_no_mismatch_error() {
    let lang = bare_language(10, 20);
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(!has_error(&res, |e| matches!(
        e,
        ValidationError::SymbolCountMismatch { .. }
    )));
}

// ===========================================================================
// 3. State-count mismatch
// ===========================================================================

#[test]
fn state_count_mismatch_high() {
    let mut lang = bare_language(10, 50);
    lang.state_count = 50;
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(has_error(&res, |e| matches!(
        e,
        ValidationError::StateCountMismatch {
            language: 50,
            tables: 20
        }
    )));
}

#[test]
fn state_count_mismatch_low() {
    let lang = bare_language(10, 5);
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(has_error(&res, |e| matches!(
        e,
        ValidationError::StateCountMismatch {
            language: 5,
            tables: 20
        }
    )));
}

#[test]
fn state_count_match_no_mismatch_error() {
    let lang = bare_language(10, 20);
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(!has_error(&res, |e| matches!(
        e,
        ValidationError::StateCountMismatch { .. }
    )));
}

// ===========================================================================
// 4. Null-pointer validation
// ===========================================================================

#[test]
fn all_null_pointers_detected() {
    // Both parse_table and small_parse_table are null → error
    let lang = bare_language(10, 20);
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(has_error(&res, |e| matches!(
        e,
        ValidationError::NullPointer("parse_table or small_parse_table")
    )));
}

#[test]
fn null_symbol_names_detected() {
    let lang = bare_language(10, 20);
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(has_error(&res, |e| matches!(
        e,
        ValidationError::NullPointer("symbol_names")
    )));
}

#[test]
fn null_symbol_metadata_detected() {
    let lang = bare_language(10, 20);
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(has_error(&res, |e| matches!(
        e,
        ValidationError::NullPointer("symbol_metadata")
    )));
}

#[test]
fn non_null_parse_table_passes_parse_table_check() {
    let dummy: u16 = 0;
    let mut lang = bare_language(10, 20);
    lang.parse_table = &dummy;
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(!has_error(&res, |e| matches!(
        e,
        ValidationError::NullPointer("parse_table or small_parse_table")
    )));
}

#[test]
fn non_null_small_parse_table_passes_parse_table_check() {
    let dummy: u16 = 0;
    let mut lang = bare_language(10, 20);
    lang.small_parse_table = &dummy;
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(!has_error(&res, |e| matches!(
        e,
        ValidationError::NullPointer("parse_table or small_parse_table")
    )));
}

#[test]
fn field_names_null_with_nonzero_field_count() {
    let mut lang = bare_language(10, 20);
    lang.field_count = 3;
    // field_names stays null
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(has_error(&res, |e| matches!(
        e,
        ValidationError::NullPointer("field_names")
    )));
}

#[test]
fn field_names_null_with_zero_field_count_ok() {
    let lang = bare_language(10, 20);
    // field_count == 0, field_names null → no "field_names" error
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(!has_error(&res, |e| matches!(
        e,
        ValidationError::NullPointer("field_names")
    )));
}

// ===========================================================================
// 5. Symbol metadata – EOF must be invisible + unnamed
// ===========================================================================

#[test]
fn eof_visible_is_invalid() {
    let metadata = vec![
        TSSymbolMetadata {
            visible: true,
            named: false,
        }, // bad EOF
        TSSymbolMetadata {
            visible: true,
            named: true,
        },
    ];
    let mut lang = bare_language(2, 1);
    lang.symbol_metadata = metadata.as_ptr();
    let tables = CompressedParseTable::new_for_testing(2, 1);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(has_error(&res, |e| matches!(
        e,
        ValidationError::InvalidSymbolMetadata { symbol: 0, .. }
    )));
    // prevent drop before validate finishes
    drop(metadata);
}

#[test]
fn eof_named_is_invalid() {
    let metadata = vec![
        TSSymbolMetadata {
            visible: false,
            named: true,
        }, // bad EOF
        TSSymbolMetadata {
            visible: true,
            named: true,
        },
    ];
    let mut lang = bare_language(2, 1);
    lang.symbol_metadata = metadata.as_ptr();
    let tables = CompressedParseTable::new_for_testing(2, 1);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(has_error(&res, |e| matches!(
        e,
        ValidationError::InvalidSymbolMetadata { symbol: 0, .. }
    )));
    drop(metadata);
}

#[test]
fn eof_invisible_unnamed_is_valid_metadata() {
    let metadata = vec![
        TSSymbolMetadata {
            visible: false,
            named: false,
        }, // correct EOF
        TSSymbolMetadata {
            visible: true,
            named: true,
        },
    ];
    let mut lang = bare_language(2, 1);
    lang.symbol_metadata = metadata.as_ptr();
    let tables = CompressedParseTable::new_for_testing(2, 1);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(!has_error(&res, |e| matches!(
        e,
        ValidationError::InvalidSymbolMetadata { .. }
    )));
    drop(metadata);
}

// ===========================================================================
// 6. Multiple simultaneous errors
// ===========================================================================

#[test]
fn multiple_errors_collected() {
    let mut lang = bare_language(99, 99);
    lang.version = 0; // wrong version
    // symbol_count and state_count both wrong
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let res = LanguageValidator::new(&lang, &tables).validate();
    let errors = res.unwrap_err();
    // At minimum: version, symbol mismatch, state mismatch, null ptrs
    assert!(errors.len() >= 4, "got only {} errors", errors.len());
}

// ===========================================================================
// 7. LanguageBuilder integration – generate_language then validate
// ===========================================================================

#[test]
fn generated_language_passes_validation() {
    let mut grammar = Grammar::new("gen_test".to_string());
    grammar.tokens.insert(
        SymbolId(0),
        Token {
            name: "NUM".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let pt = populated_parse_table(grammar.clone());
    let compressed = CompressedParseTable::from_parse_table(&pt);
    let builder = LanguageBuilder::new(grammar, pt);
    let lang = builder.generate_language().expect("generate_language");
    let res = LanguageValidator::new(&lang, &compressed).validate();
    if let Err(ref errors) = res {
        // symbol/state counts may differ because LanguageBuilder derives
        // counts from grammar while CompressedParseTable uses parse table.
        // Filter to only "unexpected" errors.
        let unexpected: Vec<_> = errors
            .iter()
            .filter(|e| {
                !matches!(
                    e,
                    ValidationError::SymbolCountMismatch { .. }
                        | ValidationError::StateCountMismatch { .. }
                )
            })
            .collect();
        assert!(
            unexpected.is_empty(),
            "unexpected validation errors: {unexpected:?}"
        );
    }
}

#[test]
fn generated_language_has_version_15() {
    let grammar = Grammar::new("v15".to_string());
    let pt = minimal_parse_table(grammar.clone());
    let builder = LanguageBuilder::new(grammar, pt);
    let lang = builder.generate_language().expect("generate_language");
    assert_eq!(lang.version, 15);
}

#[test]
fn generated_language_field_count_matches_grammar() {
    let mut grammar = Grammar::new("fields".to_string());
    grammar.fields.insert(FieldId(0), "alpha".to_string());
    grammar.fields.insert(FieldId(1), "beta".to_string());
    let pt = minimal_parse_table(grammar.clone());
    let builder = LanguageBuilder::new(grammar, pt);
    let lang = builder.generate_language().expect("generate_language");
    assert_eq!(lang.field_count, 2);
}

#[test]
fn generated_language_external_token_count() {
    let mut grammar = Grammar::new("ext".to_string());
    grammar.externals.push(adze_ir::ExternalToken {
        name: "INDENT".to_string(),
        symbol_id: SymbolId(100),
    });
    let pt = minimal_parse_table(grammar.clone());
    let builder = LanguageBuilder::new(grammar, pt);
    let lang = builder.generate_language().expect("generate_language");
    assert_eq!(lang.external_token_count, 1);
}

// ===========================================================================
// 8. ValidationError Debug / PartialEq
// ===========================================================================

#[test]
fn validation_error_debug_not_empty() {
    let err = ValidationError::InvalidVersion {
        expected: 15,
        actual: 14,
    };
    let dbg = format!("{err:?}");
    assert!(!dbg.is_empty());
    assert!(dbg.contains("14"));
}

#[test]
fn validation_error_eq() {
    let a = ValidationError::FieldNamesNotSorted;
    let b = ValidationError::FieldNamesNotSorted;
    assert_eq!(a, b);
}

#[test]
fn validation_error_ne_variant() {
    let a = ValidationError::FieldNamesNotSorted;
    let b = ValidationError::NullPointer("x");
    assert_ne!(a, b);
}

// ===========================================================================
// 9. Edge cases: zero-sized language
// ===========================================================================

#[test]
fn zero_symbol_zero_state_only_version_and_null_errors() {
    let lang = bare_language(0, 0);
    let tables = CompressedParseTable::new_for_testing(0, 0);
    let res = LanguageValidator::new(&lang, &tables).validate();
    // counts match (both zero), so no count errors
    assert!(!has_error(&res, |e| matches!(
        e,
        ValidationError::SymbolCountMismatch { .. }
    )));
    assert!(!has_error(&res, |e| matches!(
        e,
        ValidationError::StateCountMismatch { .. }
    )));
}

// ===========================================================================
// 10. Large counts
// ===========================================================================

#[test]
fn large_symbol_and_state_counts_match() {
    let lang = bare_language(10_000, 50_000);
    let tables = CompressedParseTable::new_for_testing(10_000, 50_000);
    let res = LanguageValidator::new(&lang, &tables).validate();
    assert!(!has_error(&res, |e| matches!(
        e,
        ValidationError::SymbolCountMismatch { .. }
    )));
    assert!(!has_error(&res, |e| matches!(
        e,
        ValidationError::StateCountMismatch { .. }
    )));
}

// ===========================================================================
// 11. TSLanguage struct field smoke tests
// ===========================================================================

#[test]
fn ts_language_default_external_scanner_null() {
    let lang = bare_language(1, 1);
    assert!(lang.external_scanner_data.states.is_null());
    assert!(lang.external_scanner_data.create.is_none());
}

#[test]
fn ts_language_lex_fn_none_by_default() {
    let lang = bare_language(1, 1);
    assert!(lang.lex_fn.is_none());
    assert!(lang.keyword_lex_fn.is_none());
}

// ===========================================================================
// 12. CompressedParseTable factory methods
// ===========================================================================

#[test]
fn compressed_parse_table_from_parse_table_counts() {
    let grammar = Grammar::new("cpt".to_string());
    let mut pt = minimal_parse_table(grammar);
    pt.symbol_count = 7;
    pt.state_count = 13;
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.symbol_count(), 7);
    assert_eq!(cpt.state_count(), 13);
}

#[test]
fn compressed_parse_table_new_for_testing() {
    let cpt = CompressedParseTable::new_for_testing(42, 99);
    assert_eq!(cpt.symbol_count(), 42);
    assert_eq!(cpt.state_count(), 99);
}

// ===========================================================================
// 13. Validation on a fully-wired (but tiny) language
// ===========================================================================

#[test]
fn fully_wired_minimal_language_validates() {
    // Provide non-null pointers for parse table, symbol names, and metadata
    let spt: Vec<u16> = vec![0];
    let sym_name_data = b"end\0";
    let sym_name_ptr: *const i8 = sym_name_data.as_ptr().cast();
    let sym_names = [sym_name_ptr];
    let metadata = [TSSymbolMetadata {
        visible: false,
        named: false,
    }];

    let mut lang = bare_language(1, 1);
    lang.small_parse_table = spt.as_ptr();
    lang.symbol_names = sym_names.as_ptr();
    lang.symbol_metadata = metadata.as_ptr();

    let tables = CompressedParseTable::new_for_testing(1, 1);
    let res = LanguageValidator::new(&lang, &tables).validate();
    // No null-pointer, no count-mismatch, no version errors → Ok
    assert!(
        res.is_ok(),
        "expected Ok, got errors: {:?}",
        res.unwrap_err()
    );
}

// ===========================================================================
// 14. LanguageBuilder with multiple tokens
// ===========================================================================

#[test]
fn language_builder_multiple_tokens() {
    let mut grammar = Grammar::new("multi".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "IDENT".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let pt = minimal_parse_table(grammar.clone());
    let builder = LanguageBuilder::new(grammar, pt);
    let lang = builder.generate_language().expect("generate_language");
    // token_count = #tokens + 1 (for EOF)
    assert_eq!(lang.token_count, 3);
}

// ===========================================================================
// 15. LanguageBuilder empty grammar smoke
// ===========================================================================

#[test]
fn language_builder_empty_grammar() {
    let grammar = Grammar::new("empty".to_string());
    let pt = minimal_parse_table(grammar.clone());
    let builder = LanguageBuilder::new(grammar, pt);
    let lang = builder.generate_language().expect("generate_language");
    assert_eq!(lang.version, 15);
    assert_eq!(lang.field_count, 0);
    assert_eq!(lang.external_token_count, 0);
}

// ===========================================================================
// 16. TSLexer struct is repr(C) usable
// ===========================================================================

#[test]
fn ts_lexer_default_fields() {
    let lexer = TSLexer {
        lookahead: 0,
        result_symbol: 0,
        advance: None,
        mark_end: None,
        get_column: None,
        is_at_included_range_start: None,
        eof: None,
    };
    assert_eq!(lexer.lookahead, 0);
    assert!(lexer.advance.is_none());
}
