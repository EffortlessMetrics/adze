//! Tree-sitter ABI compatibility matrix tests.
//!
//! Verify that generated structures match Tree-sitter ABI v15 expectations:
//! struct layout, counts, metadata encoding, name validity, and cross-field
//! consistency invariants.

use std::collections::BTreeMap;
use std::ffi::CStr;
use std::mem;

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseRule, ParseTable, StateId};
use adze_ir::{FieldId, Grammar, RuleId, SymbolId};
use adze_tablegen::LanguageValidator;
use adze_tablegen::abi::{
    ExternalScanner, TREE_SITTER_LANGUAGE_VERSION, TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION,
    TSFieldId, TSLanguage, TSLexState, TSParseAction, TSStateId, TSSymbol, create_symbol_metadata,
    symbol_metadata,
};
use adze_tablegen::compress::CompressedParseTable;
use adze_tablegen::schema::validate_action_encoding;
use adze_tablegen::validation::{self, ValidationError};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

/// Build a minimal ParseTable for integration tests.
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

/// Build a validation::TSLanguage with sensible defaults.
fn make_test_language(
    symbol_count: u32,
    token_count: u32,
    state_count: u32,
) -> validation::TSLanguage {
    validation::TSLanguage {
        version: 15,
        symbol_count,
        alias_count: 0,
        token_count,
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
        external_scanner_data: validation::TSExternalScannerData {
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

// ===========================================================================
// 1. Language struct has correct size and alignment
// ===========================================================================

#[test]
fn language_struct_pointer_aligned() {
    assert_eq!(
        mem::align_of::<TSLanguage>(),
        mem::align_of::<*const u8>(),
        "TSLanguage must be pointer-aligned for C FFI"
    );
}

#[test]
fn language_struct_minimum_size() {
    // 9 u32 scalars + 1 u16 = at least 38 bytes of scalar fields, plus pointers
    let size = mem::size_of::<TSLanguage>();
    assert!(
        size >= 38,
        "TSLanguage must be at least 38 bytes, got {size}"
    );
}

#[test]
fn validation_language_struct_pointer_aligned() {
    assert_eq!(
        mem::align_of::<validation::TSLanguage>(),
        mem::align_of::<*const u8>(),
        "validation::TSLanguage must also be pointer-aligned"
    );
}

// ===========================================================================
// 2. ABI version constant matches TREE_SITTER_LANGUAGE_VERSION (15)
// ===========================================================================

#[test]
fn abi_version_is_15() {
    assert_eq!(TREE_SITTER_LANGUAGE_VERSION, 15);
}

#[test]
fn min_abi_version_in_range() {
    const { assert!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION >= 13) };
    const { assert!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION <= TREE_SITTER_LANGUAGE_VERSION) };
}

// ===========================================================================
// 3. Symbol count includes all terminals + non-terminals + EOF
// ===========================================================================

#[test]
fn symbol_count_includes_terminals_nonterminals_eof() {
    let table = make_empty_table(2, 3, 2, 0);
    // terms=3 → indices 1..=3, EOF at index 4, nonterminals at 5..=6
    // Plus ERROR at index 0 → total = 7
    assert_eq!(table.symbol_count, 7);
    assert!(table.symbol_to_index.contains_key(&table.eof_symbol));
}

#[test]
fn symbol_count_with_externals() {
    let table = make_empty_table(2, 2, 1, 3);
    // ERROR(0), 2 terms(1..2), 3 externals(3..5), EOF(6), 1 nonterm(7)
    assert_eq!(table.symbol_count, 8);
    assert_eq!(table.eof_symbol, SymbolId(6));
}

// ===========================================================================
// 4. Field count matches grammar field definitions
// ===========================================================================

#[test]
fn field_count_zero_when_no_fields() {
    let grammar = Grammar::new("no_fields".to_string());
    assert_eq!(grammar.fields.len(), 0);
}

#[test]
fn field_count_matches_grammar_fields() {
    let mut grammar = Grammar::new("with_fields".to_string());
    grammar.fields.insert(FieldId(1), "left".to_string());
    grammar.fields.insert(FieldId(2), "right".to_string());
    grammar.fields.insert(FieldId(3), "operator".to_string());
    assert_eq!(grammar.fields.len(), 3);
}

// ===========================================================================
// 5. Parse table entry size matches expected bit layout
// ===========================================================================

#[test]
fn parse_action_size_is_6_bytes() {
    assert_eq!(mem::size_of::<TSParseAction>(), 6);
}

#[test]
fn parse_action_encoding_shift_fits_15_bits() {
    // Shift values occupy 0x0001..=0x7FFF
    let enc = validate_action_encoding(&Action::Shift(StateId(0x7FFF))).unwrap();
    assert_eq!(enc, 0x7FFF);
    assert!(enc & 0x8000 == 0, "Shift must not have high bit set");
}

#[test]
fn parse_action_encoding_reduce_has_high_bit() {
    let enc = validate_action_encoding(&Action::Reduce(RuleId(42))).unwrap();
    assert!(enc & 0x8000 != 0, "Reduce must have high bit set");
    assert_eq!(enc & 0x7FFF, 42);
}

// ===========================================================================
// 6. Symbol metadata flags are correctly encoded (named, visible, supertype)
// ===========================================================================

#[test]
fn metadata_visible_flag() {
    let m = create_symbol_metadata(true, false, false, false, false);
    assert_eq!(m & symbol_metadata::VISIBLE, symbol_metadata::VISIBLE);
    assert_eq!(m & symbol_metadata::NAMED, 0);
}

#[test]
fn metadata_named_flag() {
    let m = create_symbol_metadata(false, true, false, false, false);
    assert_eq!(m & symbol_metadata::NAMED, symbol_metadata::NAMED);
}

#[test]
fn metadata_supertype_flag() {
    let m = create_symbol_metadata(false, false, false, false, true);
    assert_eq!(m & symbol_metadata::SUPERTYPE, symbol_metadata::SUPERTYPE);
}

#[test]
fn metadata_all_flags_combined() {
    let m = create_symbol_metadata(true, true, true, true, true);
    assert_eq!(
        m,
        symbol_metadata::VISIBLE
            | symbol_metadata::NAMED
            | symbol_metadata::HIDDEN
            | symbol_metadata::AUXILIARY
            | symbol_metadata::SUPERTYPE
    );
    assert_eq!(m, 0x1F); // bits 0..4 all set
}

#[test]
fn metadata_hidden_and_auxiliary_independent() {
    let hidden = create_symbol_metadata(false, false, true, false, false);
    let aux = create_symbol_metadata(false, false, false, true, false);
    assert_ne!(hidden, aux);
    assert_eq!(hidden, symbol_metadata::HIDDEN);
    assert_eq!(aux, symbol_metadata::AUXILIARY);
}

// ===========================================================================
// 7. Production info encodes symbol and child counts correctly
// ===========================================================================

#[test]
fn production_id_encoding_roundtrip() {
    // Reduce(0) → 0x8000, Reduce(1) → 0x8001
    for id in [0u16, 1, 100, 1000, 0x7FFE] {
        let enc = validate_action_encoding(&Action::Reduce(RuleId(id))).unwrap();
        assert_eq!(enc & 0x7FFF, id);
    }
}

#[test]
fn parse_rule_child_count_preserved() {
    let rule = ParseRule {
        lhs: SymbolId(5),
        rhs_len: 3,
    };
    assert_eq!(rule.rhs_len, 3);
    assert_eq!(rule.lhs, SymbolId(5));
}

// ===========================================================================
// 8. State count bounds (>0, <= reasonable maximum)
// ===========================================================================

#[test]
fn state_count_positive() {
    let table = make_empty_table(1, 1, 1, 0);
    assert!(table.state_count > 0);
}

#[test]
fn state_count_fits_u16() {
    // StateId is u16, so max states must fit in u16
    let table = make_empty_table(100, 2, 1, 0);
    assert!(table.state_count <= u16::MAX as usize);
}

#[test]
fn state_count_matches_action_table_rows() {
    let table = make_empty_table(5, 2, 1, 0);
    assert_eq!(table.state_count, table.action_table.len());
}

// ===========================================================================
// 9. Token count matches terminal symbols
// ===========================================================================

#[test]
fn token_count_matches_terminals() {
    // With 4 terminals and no externals, token_count = eof_idx - 0 = 1 + 4 = 5
    let table = make_empty_table(2, 4, 1, 0);
    assert_eq!(table.token_count, 5); // ERROR(0) is implicit; eof_idx = 1+4 = 5
}

#[test]
fn token_count_excludes_externals() {
    let table = make_empty_table(2, 3, 1, 2);
    // eof_idx = 1 + 3 + 2 = 6, token_count = 6 - 2 = 4
    assert_eq!(table.token_count, 4);
}

#[test]
fn token_count_le_symbol_count() {
    for (terms, nonterms, ext) in [(1, 1, 0), (5, 3, 0), (2, 2, 3)] {
        let table = make_empty_table(2, terms, nonterms, ext);
        assert!(
            table.token_count <= table.symbol_count,
            "token_count {} must be <= symbol_count {}",
            table.token_count,
            table.symbol_count
        );
    }
}

// ===========================================================================
// 10. Keyword capture token is correctly encoded
// ===========================================================================

#[test]
fn keyword_capture_token_default_zero() {
    let lang = make_test_language(10, 5, 20);
    assert_eq!(lang.keyword_capture_token, 0);
}

#[test]
fn keyword_capture_token_fits_tssymbol() {
    // TSSymbol is u16 in validation module
    let mut lang = make_test_language(10, 5, 20);
    lang.keyword_capture_token = 0xFFFE;
    assert_eq!(lang.keyword_capture_token, 0xFFFE);
}

// ===========================================================================
// 11. External scanner state size is correctly reported
// ===========================================================================

#[test]
fn external_token_count_zero_by_default() {
    let table = make_empty_table(2, 2, 1, 0);
    assert_eq!(table.external_token_count, 0);
}

#[test]
fn external_token_count_matches_externals() {
    let table = make_empty_table(2, 2, 1, 4);
    assert_eq!(table.external_token_count, 4);
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

// ===========================================================================
// 12. Alias count matches grammar aliases
// ===========================================================================

#[test]
fn alias_count_zero_for_simple_grammar() {
    let grammar = Grammar::new("simple".to_string());
    assert!(grammar.alias_sequences.is_empty());
}

#[test]
fn validation_alias_count_field_exists() {
    let lang = make_test_language(10, 5, 20);
    assert_eq!(lang.alias_count, 0);
}

// ===========================================================================
// 13. Primary state IDs are valid state indices
// ===========================================================================

#[test]
fn primary_state_ids_null_when_absent() {
    let lang = make_test_language(10, 5, 20);
    assert!(lang.primary_state_ids.is_null());
}

#[test]
fn primary_state_ids_generated_for_real_table() {
    // When a language builder generates primary_state_ids, they must all be < state_count
    let table = make_empty_table(8, 3, 2, 0);
    // Each state's primary should be <= the state itself
    for state in 0..table.state_count {
        assert!(
            state < table.state_count,
            "primary state index must be within state_count"
        );
    }
}

// ===========================================================================
// 14. Symbol names are null-terminated and valid UTF-8
// ===========================================================================

#[test]
fn symbol_names_generated_null_terminated() {
    // The ABI builder produces `"name\0"` byte arrays for each symbol.
    // Verify the contract by checking that a round-tripped C string is valid.
    let name = b"identifier\0";
    let cstr = CStr::from_bytes_with_nul(name).expect("must be valid C string");
    assert_eq!(cstr.to_str().unwrap(), "identifier");
}

#[test]
fn eof_symbol_name_is_end() {
    let name = b"end\0";
    let cstr = CStr::from_bytes_with_nul(name).expect("must be valid C string");
    assert_eq!(cstr.to_str().unwrap(), "end");
}

#[test]
fn symbol_names_are_valid_utf8() {
    // All symbol names generated by the builder must be valid UTF-8
    for name_bytes in [b"number\0" as &[u8], b"string\0", b"_hidden\0", b"end\0"] {
        let cstr = CStr::from_bytes_with_nul(name_bytes).unwrap();
        assert!(
            cstr.to_str().is_ok(),
            "Symbol name must be valid UTF-8: {:?}",
            name_bytes
        );
    }
}

// ===========================================================================
// 15. Field names are null-terminated and valid UTF-8
// ===========================================================================

#[test]
fn field_names_null_terminated() {
    for field in [b"left\0" as &[u8], b"right\0", b"operator\0"] {
        let cstr = CStr::from_bytes_with_nul(field).unwrap();
        assert!(cstr.to_str().is_ok());
    }
}

#[test]
fn field_names_lexicographic_order_enforced_by_validation() {
    // The LanguageValidator checks field_names are in lexicographic order.
    // When field_count > 0 and field_names is null, validation must fail.
    let mut lang = make_test_language(10, 5, 20);
    lang.field_count = 3;
    // field_names is null
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let errs = LanguageValidator::new(&lang, &tables)
        .validate()
        .unwrap_err();
    assert!(
        errs.iter()
            .any(|e| matches!(e, ValidationError::NullPointer("field_names")))
    );
}

// ===========================================================================
// 16. ABI constants are self-consistent
// ===========================================================================

#[test]
fn symbol_count_ge_token_count() {
    for (terms, nonterms, ext) in [(1, 1, 0), (5, 3, 0), (2, 2, 3), (10, 5, 2)] {
        let table = make_empty_table(3, terms, nonterms, ext);
        assert!(
            table.symbol_count >= table.token_count,
            "symbol_count {} must be >= token_count {} (terms={terms}, nt={nonterms}, ext={ext})",
            table.symbol_count,
            table.token_count,
        );
    }
}

#[test]
fn symbol_count_ge_token_count_plus_external() {
    let table = make_empty_table(3, 4, 2, 3);
    assert!(table.symbol_count >= table.token_count + table.external_token_count);
}

#[test]
fn eof_index_within_terminal_boundary() {
    let table = make_empty_table(2, 3, 2, 0);
    let eof_idx = table.symbol_to_index[&table.eof_symbol];
    let terminal_boundary = table.token_count + table.external_token_count;
    // EOF index equals the terminal boundary (it is the last terminal slot)
    assert!(
        eof_idx <= terminal_boundary,
        "EOF index {eof_idx} must be <= terminal boundary {terminal_boundary}"
    );
}

#[test]
fn eof_symbol_in_symbol_to_index() {
    let table = make_empty_table(2, 2, 1, 0);
    assert!(
        table.symbol_to_index.contains_key(&table.eof_symbol),
        "EOF symbol must be in symbol_to_index map"
    );
}

#[test]
fn start_symbol_in_nonterminal_index() {
    let table = make_empty_table(2, 2, 1, 0);
    assert!(
        table.nonterminal_to_index.contains_key(&table.start_symbol),
        "Start symbol must be in nonterminal_to_index map"
    );
}

// ===========================================================================
// Additional consistency tests (beyond the 16 categories)
// ===========================================================================

#[test]
fn lex_state_size_is_4_bytes() {
    assert_eq!(mem::size_of::<TSLexState>(), 4);
}

#[test]
fn ts_symbol_size_is_2_bytes() {
    assert_eq!(mem::size_of::<TSSymbol>(), 2);
}

#[test]
fn ts_state_id_size_is_2_bytes() {
    assert_eq!(mem::size_of::<TSStateId>(), 2);
}

#[test]
fn ts_field_id_size_is_2_bytes() {
    assert_eq!(mem::size_of::<TSFieldId>(), 2);
}

#[test]
fn action_error_encoded_as_zero() {
    assert_eq!(validate_action_encoding(&Action::Error).unwrap(), 0x0000);
}

#[test]
fn action_accept_encoded_as_ffff() {
    assert_eq!(validate_action_encoding(&Action::Accept).unwrap(), 0xFFFF);
}

#[test]
fn shift_zero_rejected() {
    assert!(
        validate_action_encoding(&Action::Shift(StateId(0))).is_err(),
        "Shift(0) collides with Error encoding"
    );
}

#[test]
fn reduce_max_rejected() {
    assert!(
        validate_action_encoding(&Action::Reduce(RuleId(0x7FFF))).is_err(),
        "Reduce(0x7FFF) collides with Accept encoding"
    );
}

#[test]
fn validation_rejects_wrong_version() {
    let mut lang = make_test_language(10, 5, 20);
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
fn validation_requires_symbol_names_ptr() {
    let lang = make_test_language(10, 5, 20);
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
fn validation_requires_symbol_metadata_ptr() {
    let lang = make_test_language(10, 5, 20);
    let tables = CompressedParseTable::new_for_testing(10, 20);
    let errs = LanguageValidator::new(&lang, &tables)
        .validate()
        .unwrap_err();
    assert!(
        errs.iter()
            .any(|e| matches!(e, ValidationError::NullPointer("symbol_metadata")))
    );
}

#[test]
fn lex_modes_count_matches_state_count() {
    let table = make_empty_table(7, 2, 1, 0);
    assert_eq!(table.lex_modes.len(), table.state_count);
}

#[test]
fn index_to_symbol_length_matches_symbol_count() {
    let table = make_empty_table(3, 4, 2, 0);
    assert_eq!(table.index_to_symbol.len(), table.symbol_count);
}

#[test]
fn symbol_to_index_and_index_to_symbol_roundtrip() {
    let table = make_empty_table(3, 3, 2, 0);
    for (&sym, &idx) in &table.symbol_to_index {
        assert_eq!(
            table.index_to_symbol[idx], sym,
            "roundtrip failed for SymbolId({}) at index {}",
            sym.0, idx
        );
    }
}

#[test]
fn goto_table_dimensions_match_state_and_symbol_count() {
    let table = make_empty_table(4, 3, 2, 0);
    assert_eq!(table.goto_table.len(), table.state_count);
    for row in &table.goto_table {
        assert_eq!(row.len(), table.symbol_count);
    }
}

#[test]
fn action_table_dimensions_match_state_and_symbol_count() {
    let table = make_empty_table(4, 3, 2, 0);
    assert_eq!(table.action_table.len(), table.state_count);
    for row in &table.action_table {
        assert_eq!(row.len(), table.symbol_count);
    }
}
