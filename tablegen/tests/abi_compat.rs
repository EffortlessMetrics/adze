//! ABI compatibility integration tests for the tablegen crate.
//!
//! These tests verify ABI correctness through the public API only:
//! struct layout, encoding roundtrips, validation, and version constants.

use std::mem;

// ---------------------------------------------------------------------------
// 1. TSLanguage struct layout: sizes and alignment (abi module)
// ---------------------------------------------------------------------------

#[test]
fn abi_tslanguage_pointer_aligned() {
    use adze_tablegen::abi::TSLanguage;
    assert_eq!(
        mem::align_of::<TSLanguage>(),
        mem::align_of::<*const u8>(),
        "TSLanguage must be pointer-aligned for FFI"
    );
}

#[test]
fn abi_primitive_type_sizes() {
    use adze_tablegen::abi::{TSFieldId, TSLexState, TSParseAction, TSStateId, TSSymbol};
    assert_eq!(mem::size_of::<TSSymbol>(), 2);
    assert_eq!(mem::size_of::<TSStateId>(), 2);
    assert_eq!(mem::size_of::<TSFieldId>(), 2);
    assert_eq!(mem::size_of::<TSParseAction>(), 6);
    assert_eq!(mem::size_of::<TSLexState>(), 4);
}

#[test]
fn validation_tslanguage_pointer_aligned() {
    use adze_tablegen::validation::TSLanguage;
    assert_eq!(
        mem::align_of::<TSLanguage>(),
        mem::align_of::<*const u8>(),
        "validation::TSLanguage must be pointer-aligned for FFI"
    );
}

#[test]
fn both_tslanguage_structs_large_enough_for_abi_v15() {
    let abi_size = mem::size_of::<adze_tablegen::abi::TSLanguage>();
    let val_size = mem::size_of::<adze_tablegen::validation::TSLanguage>();
    // Minimum: 9 × u32 + 1 × u16 = 38 bytes of scalar fields
    let minimum = 9 * 4 + 2;
    assert!(abi_size >= minimum, "abi::TSLanguage too small");
    assert!(val_size >= minimum, "validation::TSLanguage too small");
}

// ---------------------------------------------------------------------------
// 2. ABI version constants
// ---------------------------------------------------------------------------

#[test]
fn abi_version_constants() {
    use adze_tablegen::abi::{
        TREE_SITTER_LANGUAGE_VERSION, TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION,
    };
    assert_eq!(TREE_SITTER_LANGUAGE_VERSION, 15);
    const { assert!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION <= 15) };
    const { assert!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION >= 13) };
}

// ---------------------------------------------------------------------------
// 3. Symbol metadata flag encoding roundtrip
// ---------------------------------------------------------------------------

#[test]
fn symbol_metadata_individual_flags() {
    use adze_tablegen::abi::{create_symbol_metadata, symbol_metadata};

    assert_eq!(create_symbol_metadata(false, false, false, false, false), 0);
    assert_eq!(
        create_symbol_metadata(true, false, false, false, false),
        symbol_metadata::VISIBLE
    );
    assert_eq!(
        create_symbol_metadata(false, true, false, false, false),
        symbol_metadata::NAMED
    );
    assert_eq!(
        create_symbol_metadata(false, false, true, false, false),
        symbol_metadata::HIDDEN
    );
    assert_eq!(
        create_symbol_metadata(false, false, false, true, false),
        symbol_metadata::AUXILIARY
    );
    assert_eq!(
        create_symbol_metadata(false, false, false, false, true),
        symbol_metadata::SUPERTYPE
    );
}

#[test]
fn symbol_metadata_combined_flags() {
    use adze_tablegen::abi::{create_symbol_metadata, symbol_metadata};

    let m = create_symbol_metadata(true, true, false, false, false);
    assert_eq!(m, symbol_metadata::VISIBLE | symbol_metadata::NAMED);

    let all = create_symbol_metadata(true, true, true, true, true);
    assert_eq!(
        all,
        symbol_metadata::VISIBLE
            | symbol_metadata::NAMED
            | symbol_metadata::HIDDEN
            | symbol_metadata::AUXILIARY
            | symbol_metadata::SUPERTYPE
    );
}

// ---------------------------------------------------------------------------
// 4. Parse table action encoding/decoding roundtrip (schema module)
// ---------------------------------------------------------------------------

#[test]
fn action_encoding_roundtrip_shift() {
    use adze_glr_core::{Action, StateId};
    use adze_tablegen::schema::validate_action_encoding;

    for s in [1u16, 2, 100, 0x7FFE] {
        let enc = validate_action_encoding(&Action::Shift(StateId(s))).unwrap();
        assert_eq!(enc, s, "Shift({s}) must encode to {s}");
        assert!(enc < 0x8000, "Shift must not have high bit");
    }
}

#[test]
fn action_encoding_roundtrip_reduce() {
    use adze_glr_core::Action;
    use adze_ir::RuleId;
    use adze_tablegen::schema::validate_action_encoding;

    for p in [0u16, 1, 100, 0x7FFD] {
        let enc = validate_action_encoding(&Action::Reduce(RuleId(p))).unwrap();
        assert!(enc & 0x8000 != 0, "Reduce must have high bit");
        assert_eq!(enc & 0x7FFF, p, "Reduce payload must match");
    }
}

#[test]
fn action_encoding_error_and_accept() {
    use adze_glr_core::Action;
    use adze_tablegen::schema::validate_action_encoding;

    assert_eq!(validate_action_encoding(&Action::Error).unwrap(), 0x0000);
    assert_eq!(validate_action_encoding(&Action::Accept).unwrap(), 0xFFFF);
}

#[test]
fn action_encoding_shift_zero_rejected() {
    use adze_glr_core::{Action, StateId};
    use adze_tablegen::schema::validate_action_encoding;
    assert!(
        validate_action_encoding(&Action::Shift(StateId(0))).is_err(),
        "Shift(0) must be rejected (collides with Error)"
    );
}

#[test]
fn action_encoding_reduce_max_rejected() {
    use adze_glr_core::Action;
    use adze_ir::RuleId;
    use adze_tablegen::schema::validate_action_encoding;
    assert!(
        validate_action_encoding(&Action::Reduce(RuleId(0x7FFF))).is_err(),
        "Reduce(0x7FFF) must be rejected (collides with Accept)"
    );
}

// ---------------------------------------------------------------------------
// 5. EOF symbol does not collide with grammar symbols
// ---------------------------------------------------------------------------

#[test]
fn eof_symbol_no_collision() {
    use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
    use adze_ir::*;

    let mut grammar = Grammar::new("eof_test".to_string());
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );

    let start = SymbolId(2);
    grammar.rules.entry(start).or_default().push(Rule {
        lhs: start,
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();

    let user_ids: Vec<u16> = grammar
        .tokens
        .keys()
        .chain(grammar.rules.keys())
        .map(|s| s.0)
        .collect();
    assert!(
        !user_ids.contains(&table.eof_symbol.0),
        "EOF {:?} must not collide with grammar symbols {:?}",
        table.eof_symbol,
        user_ids
    );
    assert!(table.symbol_to_index.contains_key(&table.eof_symbol));

    let eof_idx = table.symbol_to_index[&table.eof_symbol];
    let terminal_boundary = table.token_count + table.external_token_count;
    assert!(
        eof_idx < terminal_boundary,
        "EOF index {} must be < terminal boundary {}",
        eof_idx,
        terminal_boundary
    );
}

// ---------------------------------------------------------------------------
// 6. Validation catches wrong version and null pointers
// ---------------------------------------------------------------------------

/// Helper: construct a minimal TSLanguage with all-null pointers for validation tests.
fn make_test_language() -> adze_tablegen::validation::TSLanguage {
    use adze_tablegen::validation::*;
    TSLanguage {
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

#[test]
fn validation_rejects_wrong_version() {
    use adze_tablegen::LanguageValidator;
    use adze_tablegen::compress::CompressedParseTable;
    use adze_tablegen::validation::ValidationError;

    let mut lang = make_test_language();
    lang.version = 14;

    let tables = CompressedParseTable::new_for_testing(10, 20);
    let errs = LanguageValidator::new(&lang, &tables)
        .validate()
        .unwrap_err();
    assert!(errs.iter().any(|e| matches!(
        e,
        ValidationError::InvalidVersion {
            expected: 15,
            actual: 14
        }
    )));
}

#[test]
fn validation_requires_symbol_names() {
    use adze_tablegen::LanguageValidator;
    use adze_tablegen::compress::CompressedParseTable;
    use adze_tablegen::validation::ValidationError;

    let lang = make_test_language();
    assert!(lang.symbol_names.is_null());

    let tables = CompressedParseTable::new_for_testing(10, 20);
    let errs = LanguageValidator::new(&lang, &tables)
        .validate()
        .unwrap_err();
    assert!(
        errs.iter()
            .any(|e| matches!(e, ValidationError::NullPointer("symbol_names")))
    );
}

#[test]
fn validation_requires_symbol_metadata() {
    use adze_tablegen::LanguageValidator;
    use adze_tablegen::compress::CompressedParseTable;
    use adze_tablegen::validation::ValidationError;

    let lang = make_test_language();
    assert!(lang.symbol_metadata.is_null());

    let tables = CompressedParseTable::new_for_testing(10, 20);
    let errs = LanguageValidator::new(&lang, &tables)
        .validate()
        .unwrap_err();
    assert!(
        errs.iter()
            .any(|e| matches!(e, ValidationError::NullPointer("symbol_metadata")))
    );
}
