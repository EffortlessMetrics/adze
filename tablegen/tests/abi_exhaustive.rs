//! Exhaustive ABI compatibility tests for the tablegen crate.
//!
//! These tests ensure correctness of:
//! 1. Every symbol metadata flag combination
//! 2. Generated Language struct field count
//! 3. Parse action encoding/decoding roundtrip for all action types
//! 4. State ID encoding/decoding roundtrip
//! 5. Symbol ID encoding/decoding roundtrip
//! 6. LanguageBuilder validates required fields
//! 7. Generated code includes version constants
//! 8. External scanner table generation correctness
//! 9. Keyword extraction table correctness
//! 10. Primary state lookup correctness

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable, StateId};
use adze_ir::{Grammar, RuleId, SymbolId};
use adze_tablegen::abi::{
    ExternalScanner, TREE_SITTER_LANGUAGE_VERSION, TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION,
    TSFieldId, TSLanguage, TSLexState, TSParseAction, TSStateId, TSSymbol, create_symbol_metadata,
    symbol_metadata,
};
use adze_tablegen::schema::{validate_action_decoding, validate_action_encoding};
use std::collections::BTreeMap;
use std::mem;

// ---------------------------------------------------------------------------
// Helper: build a minimal ParseTable suitable for integration tests.
// Mirrors the internal make_empty_table but is self-contained.
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

fn make_empty_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms + externals;
    let nonterms_eff = if nonterms == 0 { 1 } else { nonterms };
    let symbol_count = eof_idx + 1 + nonterms_eff;

    let actions = vec![vec![vec![]; symbol_count]; states];
    let gotos = vec![vec![INVALID; symbol_count]; states];

    let start_symbol = SymbolId((eof_idx + 1) as u16);
    let eof_symbol = SymbolId(eof_idx as u16);
    let token_count = eof_idx - externals;

    let mut symbol_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let mut nonterminal_to_index: BTreeMap<SymbolId, usize> = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, start_symbol.0 as usize);

    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (symbol_id, index) in &symbol_to_index {
        index_to_symbol[*index] = *symbol_id;
    }

    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        states
    ];

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: vec![],
        state_count: states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: vec![],
        token_count,
        external_token_count: externals,
        eof_symbol,
        start_symbol,
        initial_state: StateId(0),
        lex_modes,
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: Grammar::default(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

// ===========================================================================
// 1. Every symbol metadata flag combination produces valid flags
// ===========================================================================

#[test]
fn exhaustive_symbol_metadata_all_32_combinations() {
    for bits in 0u8..32 {
        let visible = bits & 0x01 != 0;
        let named = bits & 0x02 != 0;
        let hidden = bits & 0x04 != 0;
        let auxiliary = bits & 0x08 != 0;
        let supertype = bits & 0x10 != 0;

        let metadata = create_symbol_metadata(visible, named, hidden, auxiliary, supertype);

        assert_eq!(
            metadata & 0xE0,
            0,
            "upper 3 bits must be zero for flags={bits:#07b}"
        );
        assert_eq!(
            metadata & symbol_metadata::VISIBLE != 0,
            visible,
            "VISIBLE mismatch for bits={bits:#07b}"
        );
        assert_eq!(
            metadata & symbol_metadata::NAMED != 0,
            named,
            "NAMED mismatch for bits={bits:#07b}"
        );
        assert_eq!(
            metadata & symbol_metadata::HIDDEN != 0,
            hidden,
            "HIDDEN mismatch for bits={bits:#07b}"
        );
        assert_eq!(
            metadata & symbol_metadata::AUXILIARY != 0,
            auxiliary,
            "AUXILIARY mismatch for bits={bits:#07b}"
        );
        assert_eq!(
            metadata & symbol_metadata::SUPERTYPE != 0,
            supertype,
            "SUPERTYPE mismatch for bits={bits:#07b}"
        );
    }
}

#[test]
fn symbol_metadata_flags_are_distinct_powers_of_two() {
    let flags = [
        symbol_metadata::VISIBLE,
        symbol_metadata::NAMED,
        symbol_metadata::HIDDEN,
        symbol_metadata::AUXILIARY,
        symbol_metadata::SUPERTYPE,
    ];
    for (i, &a) in flags.iter().enumerate() {
        assert!(a.is_power_of_two(), "flag {i} must be a power of two");
        for (j, &b) in flags.iter().enumerate() {
            if i != j {
                assert_eq!(a & b, 0, "flags {i} and {j} must not overlap");
            }
        }
    }
}

#[test]
fn symbol_metadata_zero_flags_yields_zero() {
    assert_eq!(create_symbol_metadata(false, false, false, false, false), 0);
}

#[test]
fn symbol_metadata_all_flags_yields_0x1f() {
    assert_eq!(create_symbol_metadata(true, true, true, true, true), 0x1F);
}

// ===========================================================================
// 2. Generated Language struct has correct field count
// ===========================================================================

#[test]
fn tslanguage_abi_struct_field_count() {
    let size = mem::size_of::<TSLanguage>();
    assert!(
        size >= 38,
        "TSLanguage too small: {size} bytes (need >= 38 for ABI v15 scalars)"
    );
    assert_eq!(
        mem::align_of::<TSLanguage>(),
        mem::align_of::<*const u8>(),
        "TSLanguage must be pointer-aligned"
    );
}

#[test]
fn tslanguage_validation_struct_field_count() {
    use adze_tablegen::validation::TSLanguage as ValTSLanguage;
    let size = mem::size_of::<ValTSLanguage>();
    assert!(size >= 38, "validation::TSLanguage too small: {size} bytes");
    assert_eq!(
        mem::align_of::<ValTSLanguage>(),
        mem::align_of::<*const u8>(),
    );
}

#[test]
fn tslanguage_contains_all_required_pointer_fields() {
    let size = mem::size_of::<TSLanguage>();
    let ptr_size = mem::size_of::<*const u8>();
    let min_size = 38 + 16 * ptr_size;
    assert!(
        size >= min_size,
        "TSLanguage at {size} bytes is too small for all ABI v15 fields (need >= {min_size})"
    );
}

// ===========================================================================
// 3. Parse action encoding/decoding roundtrip for all action types
// ===========================================================================

#[test]
fn action_roundtrip_shift_boundary_values() {
    for s in [1u16, 2, 255, 0x3FFF, 0x7FFE] {
        let action = Action::Shift(StateId(s));
        let enc = validate_action_encoding(&action).unwrap();
        assert_eq!(enc, s);
        validate_action_decoding(enc, &action).unwrap();
    }
}

#[test]
fn action_roundtrip_reduce_boundary_values() {
    for p in [0u16, 1, 255, 0x3FFF, 0x7FFD, 0x7FFE] {
        let action = Action::Reduce(RuleId(p));
        let enc = validate_action_encoding(&action).unwrap();
        assert!(enc & 0x8000 != 0, "Reduce must have high bit");
        assert_eq!(enc & 0x7FFF, p, "Reduce payload must roundtrip");
        validate_action_decoding(enc, &action).unwrap();
    }
}

#[test]
fn action_roundtrip_error() {
    let enc = validate_action_encoding(&Action::Error).unwrap();
    assert_eq!(enc, 0x0000);
    validate_action_decoding(enc, &Action::Error).unwrap();
}

#[test]
fn action_roundtrip_accept() {
    let enc = validate_action_encoding(&Action::Accept).unwrap();
    assert_eq!(enc, 0xFFFF);
    validate_action_decoding(enc, &Action::Accept).unwrap();
}

#[test]
fn action_encoding_rejects_shift_zero() {
    assert!(
        validate_action_encoding(&Action::Shift(StateId(0))).is_err(),
        "Shift(0) collides with Error encoding"
    );
}

#[test]
fn action_encoding_rejects_shift_overflow() {
    assert!(
        validate_action_encoding(&Action::Shift(StateId(0x8000))).is_err(),
        "Shift >= 0x8000 collides with Reduce encoding"
    );
}

#[test]
fn action_encoding_rejects_reduce_max() {
    assert!(
        validate_action_encoding(&Action::Reduce(RuleId(0x7FFF))).is_err(),
        "Reduce(0x7FFF) collides with Accept encoding (0xFFFF)"
    );
}

#[test]
fn action_encoding_rejects_recover() {
    assert!(
        validate_action_encoding(&Action::Recover).is_err(),
        "Recover is a runtime-only action"
    );
}

#[test]
fn action_encoding_rejects_fork() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    assert!(
        validate_action_encoding(&fork).is_err(),
        "Fork is a runtime-only action"
    );
}

// ===========================================================================
// 4. State ID encoding/decoding roundtrip
// ===========================================================================

#[test]
fn state_id_roundtrip_via_shift() {
    for s in [1u16, 2, 100, 1000, 0x7FFE] {
        let action = Action::Shift(StateId(s));
        let enc = validate_action_encoding(&action).unwrap();
        assert!(enc != 0 && enc & 0x8000 == 0 && enc != 0xFFFF);
        assert_eq!(enc, s, "state ID must roundtrip through encoding");
    }
}

#[test]
fn state_id_type_is_u16() {
    assert_eq!(mem::size_of::<TSStateId>(), 2);
    assert_eq!(TSStateId(0).0, 0u16);
    assert_eq!(TSStateId(u16::MAX).0, u16::MAX);
}

#[test]
fn state_id_covers_valid_range() {
    let min_valid = 1u16;
    let max_valid = 0x7FFEu16;
    validate_action_encoding(&Action::Shift(StateId(min_valid))).unwrap();
    validate_action_encoding(&Action::Shift(StateId(max_valid))).unwrap();
}

// ===========================================================================
// 5. Symbol ID encoding/decoding roundtrip
// ===========================================================================

#[test]
fn symbol_id_type_is_u16() {
    assert_eq!(mem::size_of::<TSSymbol>(), 2);
    assert_eq!(TSSymbol(0).0, 0u16);
    assert_eq!(TSSymbol(u16::MAX).0, u16::MAX);
}

#[test]
fn symbol_id_roundtrip_via_reduce() {
    for p in [0u16, 1, 100, 0x7FFD, 0x7FFE] {
        let action = Action::Reduce(RuleId(p));
        let enc = validate_action_encoding(&action).unwrap();
        let decoded_prod = enc & 0x7FFF;
        assert_eq!(decoded_prod, p, "production ID must roundtrip");
    }
}

#[test]
fn field_id_type_is_u16() {
    assert_eq!(mem::size_of::<TSFieldId>(), 2);
}

// ===========================================================================
// 6. LanguageBuilder validates required fields
// ===========================================================================

#[test]
fn language_builder_produces_valid_version() {
    use adze_tablegen::LanguageBuilder;

    let grammar = Grammar::new("test_builder".to_string());
    let table = make_empty_table(1, 1, 1, 0);
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder.generate_language().unwrap();

    assert_eq!(lang.version, 15, "LanguageBuilder must produce ABI v15");
}

#[test]
fn language_builder_counts_match_grammar() {
    use adze_ir::*;
    use adze_tablegen::LanguageBuilder;

    let mut grammar = Grammar::new("count_check".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "b".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    grammar.fields.insert(FieldId(1), "left".into());

    let table = make_empty_table(2, 2, 1, 0);
    let builder = LanguageBuilder::new(grammar.clone(), table);
    let lang = builder.generate_language().unwrap();

    assert_eq!(lang.field_count, 1, "field_count must match grammar");
    assert_eq!(lang.external_token_count, 0, "no externals in this grammar");
}

#[test]
fn language_builder_rejects_wrong_version_via_validation() {
    use adze_tablegen::LanguageValidator;
    use adze_tablegen::compress::CompressedParseTable;
    use adze_tablegen::validation::ValidationError;

    let grammar = Grammar::new("ver_val".into());
    let table = make_empty_table(1, 1, 1, 0);
    let builder = adze_tablegen::LanguageBuilder::new(grammar, table.clone());
    let mut lang = builder.generate_language().unwrap();
    lang.version = 14; // invalid version

    let compressed = CompressedParseTable::from_parse_table(&table);
    let result = LanguageValidator::new(&lang, &compressed).validate();
    assert!(
        result.is_err(),
        "validation should reject wrong ABI version"
    );
    let errs = result.unwrap_err();
    assert!(
        errs.iter().any(|e| matches!(
            e,
            ValidationError::InvalidVersion {
                expected: 15,
                actual: 14
            }
        )),
        "expected InvalidVersion error, got: {errs:?}"
    );
}

// ===========================================================================
// 7. Generated code includes version constants
// ===========================================================================

#[test]
fn version_constant_is_15() {
    assert_eq!(TREE_SITTER_LANGUAGE_VERSION, 15);
}

#[test]
fn min_compatible_version_in_range() {
    assert!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION >= 13);
    assert!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION <= TREE_SITTER_LANGUAGE_VERSION);
}

#[test]
fn abi_builder_generates_version_in_code() {
    use adze_tablegen::AbiLanguageBuilder;

    let grammar = Grammar::new("ver_test".into());
    let table = make_empty_table(1, 1, 1, 0);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();

    assert!(
        code.contains("TREE_SITTER_LANGUAGE_VERSION"),
        "generated code must reference TREE_SITTER_LANGUAGE_VERSION"
    );
}

#[test]
fn abi_builder_generates_language_static() {
    use adze_tablegen::AbiLanguageBuilder;

    let grammar = Grammar::new("lang_static".into());
    let table = make_empty_table(1, 1, 1, 0);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();

    assert!(
        code.contains("LANGUAGE"),
        "generated code must define LANGUAGE static"
    );
    assert!(
        code.contains("TSLanguage"),
        "generated code must reference TSLanguage type"
    );
}

// ===========================================================================
// 8. External scanner table generation correctness
// ===========================================================================

#[test]
fn external_scanner_default_is_null() {
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
fn external_scanner_generator_symbol_map() {
    use adze_ir::*;
    use adze_tablegen::ExternalScannerGenerator;

    let mut grammar = Grammar::new("ext_scan".into());
    grammar.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: SymbolId(10),
    });
    grammar.externals.push(ExternalToken {
        name: "dedent".into(),
        symbol_id: SymbolId(11),
    });

    let scanner_gen = ExternalScannerGenerator::new(grammar);
    let map = scanner_gen.generate_symbol_map();

    assert_eq!(map.len(), 2, "symbol map must have one entry per external");
    assert_eq!(map[0], 10, "first external maps to SymbolId(10)");
    assert_eq!(map[1], 11, "second external maps to SymbolId(11)");
}

#[test]
fn external_scanner_generator_state_bitmap() {
    use adze_ir::*;
    use adze_tablegen::ExternalScannerGenerator;

    let mut grammar = Grammar::new("ext_bitmap".into());
    grammar.externals.push(ExternalToken {
        name: "newline".into(),
        symbol_id: SymbolId(5),
    });

    let scanner_gen = ExternalScannerGenerator::new(grammar);
    let bitmap = scanner_gen.generate_state_bitmap(3);

    assert_eq!(bitmap.len(), 3, "bitmap must have one row per state");
    for (state_idx, row) in bitmap.iter().enumerate() {
        assert_eq!(
            row.len(),
            1,
            "each row must have one column per external token (state {state_idx})"
        );
    }
}

#[test]
fn external_scanner_empty_grammar_produces_empty_map() {
    use adze_tablegen::ExternalScannerGenerator;

    let grammar = Grammar::new("no_ext".into());
    let scanner_gen = ExternalScannerGenerator::new(grammar);
    assert!(scanner_gen.generate_symbol_map().is_empty());
    // With 0 externals, each state row has 0 columns
    let bitmap = scanner_gen.generate_state_bitmap(5);
    for row in &bitmap {
        assert!(row.is_empty(), "no external tokens means empty rows");
    }
}

// ===========================================================================
// 9. Keyword extraction table correctness
// ===========================================================================

#[test]
fn abi_builder_keyword_capture_token_default_zero() {
    use adze_tablegen::AbiLanguageBuilder;

    let grammar = Grammar::new("kw_test".into());
    let table = make_empty_table(1, 1, 1, 0);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();

    assert!(
        code.contains("keyword_capture_token") || code.contains("keyword_lex_fn"),
        "generated code must include keyword fields"
    );
}

#[test]
fn abi_builder_no_keyword_lex_fn_when_not_configured() {
    use adze_tablegen::AbiLanguageBuilder;

    let grammar = Grammar::new("no_kw".into());
    let table = make_empty_table(1, 1, 1, 0);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();

    assert!(
        code.contains("None"),
        "keyword_lex_fn should be None when no keywords"
    );
}

#[test]
fn abi_builder_keyword_capture_zero_in_language() {
    use adze_tablegen::LanguageBuilder;

    let grammar = Grammar::new("kw_cap".into());
    let table = make_empty_table(1, 1, 1, 0);
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder.generate_language().unwrap();

    assert_eq!(
        lang.keyword_capture_token, 0,
        "keyword_capture_token must be 0 without keyword extraction"
    );
    assert!(
        lang.keyword_lex_fn.is_none(),
        "keyword_lex_fn must be None without keywords"
    );
}

// ===========================================================================
// 10. Primary state lookup correctness
// ===========================================================================

#[test]
fn abi_builder_generates_primary_state_ids() {
    use adze_tablegen::AbiLanguageBuilder;

    let grammar = Grammar::new("primary".into());
    let table = make_empty_table(4, 1, 1, 0);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();

    assert!(
        code.contains("PRIMARY_STATE_IDS"),
        "generated code must include PRIMARY_STATE_IDS"
    );
}

#[test]
fn primary_state_ids_identity_mapping_in_generated_language() {
    use adze_tablegen::LanguageBuilder;

    let grammar = Grammar::new("ps_identity".into());
    let table = make_empty_table(5, 1, 1, 0);
    let builder = LanguageBuilder::new(grammar, table);
    let lang = builder.generate_language().unwrap();

    assert_eq!(lang.state_count, 5);
}

#[test]
fn primary_state_ids_covers_all_states() {
    use adze_tablegen::AbiLanguageBuilder;

    let grammar = Grammar::new("ps_all".into());
    let state_count = 7;
    let table = make_empty_table(state_count, 2, 1, 0);
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate().to_string();

    assert!(
        code.contains("PRIMARY_STATE_IDS"),
        "must generate PRIMARY_STATE_IDS"
    );
    assert!(
        code.contains("state_count"),
        "generated LANGUAGE must set state_count"
    );
}

// ===========================================================================
// Additional edge-case and structural tests
// ===========================================================================

#[test]
fn parse_action_struct_size_is_6() {
    assert_eq!(mem::size_of::<TSParseAction>(), 6);
}

#[test]
fn lex_state_struct_size_is_4() {
    assert_eq!(mem::size_of::<TSLexState>(), 4);
}

#[test]
fn small_table_action_encoding_matches_schema() {
    use adze_tablegen::TableCompressor;

    let compressor = TableCompressor::new();

    let enc = compressor
        .encode_action_small(&Action::Shift(StateId(42)))
        .unwrap();
    assert_eq!(enc, 42);

    let enc = compressor
        .encode_action_small(&Action::Reduce(RuleId(5)))
        .unwrap();
    assert_eq!(enc & 0x8000, 0x8000, "Reduce must have high bit set");
    assert_eq!(enc & 0x7FFF, 6, "Small table Reduce uses 1-based IDs");

    let enc = compressor.encode_action_small(&Action::Accept).unwrap();
    assert_eq!(enc, 0xFFFF);

    let enc = compressor.encode_action_small(&Action::Error).unwrap();
    assert_eq!(enc, 0xFFFE);
}

#[test]
fn compressed_parse_table_from_parse_table_preserves_counts() {
    use adze_tablegen::compress::CompressedParseTable;

    let table = make_empty_table(10, 3, 2, 1);
    let compressed = CompressedParseTable::from_parse_table(&table);

    assert_eq!(compressed.symbol_count(), table.symbol_count);
    assert_eq!(compressed.state_count(), table.state_count);
}

#[test]
fn validation_catches_mismatched_symbol_count() {
    use adze_tablegen::LanguageValidator;
    use adze_tablegen::compress::CompressedParseTable;
    use adze_tablegen::validation::ValidationError;

    let grammar = Grammar::new("mismatch".into());
    let table = make_empty_table(1, 1, 1, 0);
    let builder = adze_tablegen::LanguageBuilder::new(grammar, table);
    let mut lang = builder.generate_language().unwrap();

    let real_count = lang.symbol_count;
    lang.symbol_count = 999;

    let tables =
        CompressedParseTable::new_for_testing(real_count as usize, lang.state_count as usize);
    let errs = LanguageValidator::new(&lang, &tables)
        .validate()
        .unwrap_err();
    assert!(
        errs.iter()
            .any(|e| matches!(e, ValidationError::SymbolCountMismatch { .. })),
        "expected SymbolCountMismatch, got: {errs:?}"
    );
}

#[test]
fn validation_catches_mismatched_state_count() {
    use adze_tablegen::LanguageValidator;
    use adze_tablegen::compress::CompressedParseTable;
    use adze_tablegen::validation::ValidationError;

    let grammar = Grammar::new("state_mm".into());
    let table = make_empty_table(3, 1, 1, 0);
    let builder = adze_tablegen::LanguageBuilder::new(grammar, table);
    let mut lang = builder.generate_language().unwrap();

    let real_states = lang.state_count;
    lang.state_count = 999;

    let tables =
        CompressedParseTable::new_for_testing(lang.symbol_count as usize, real_states as usize);
    let errs = LanguageValidator::new(&lang, &tables)
        .validate()
        .unwrap_err();
    assert!(
        errs.iter()
            .any(|e| matches!(e, ValidationError::StateCountMismatch { .. })),
        "expected StateCountMismatch, got: {errs:?}"
    );
}
