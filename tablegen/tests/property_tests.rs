// Property-based tests for table compression and ABI encoding

use adze_glr_core::{Action, StateId};
use adze_ir::RuleId;
use adze_tablegen::abi::{
    TREE_SITTER_LANGUAGE_VERSION, TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION,
    create_symbol_metadata, symbol_metadata,
};
use adze_tablegen::compress::TableCompressor;
use adze_tablegen::schema::{validate_action_decoding, validate_action_encoding};
use adze_tablegen::validation::{LanguageValidator, ValidationError};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a valid Shift action (state in 1..0x7FFF to avoid Error overlap)
fn shift_action() -> impl Strategy<Value = Action> {
    (1u16..0x8000u16).prop_map(|s| Action::Shift(StateId(s)))
}

/// Generate a valid Reduce action (rule in 0..0x7FFE to avoid Accept overlap)
fn reduce_action() -> impl Strategy<Value = Action> {
    (0u16..0x7FFFu16).prop_map(|r| Action::Reduce(RuleId(r)))
}

/// Generate any encodable action (schema encoding)
fn schema_encodable_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        Just(Action::Error),
        Just(Action::Accept),
        shift_action(),
        reduce_action(),
    ]
}

/// Generate a Shift action encodable in the small-table format (state < 0x8000)
fn small_table_shift() -> impl Strategy<Value = Action> {
    (0u16..0x8000u16).prop_map(|s| Action::Shift(StateId(s)))
}

/// Generate a Reduce action encodable in the small-table format (rule < 0x4000)
fn small_table_reduce() -> impl Strategy<Value = Action> {
    (0u16..0x4000u16).prop_map(|r| Action::Reduce(RuleId(r)))
}

/// Generate any action encodable in the small-table format
fn small_table_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        Just(Action::Accept),
        Just(Action::Error),
        Just(Action::Recover),
        small_table_shift(),
        small_table_reduce(),
    ]
}

// ---------------------------------------------------------------------------
// 1. Action encoding roundtrip (schema encoding)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn schema_action_roundtrip(action in schema_encodable_action()) {
        let encoded = validate_action_encoding(&action)
            .expect("valid action must encode");
        validate_action_decoding(encoded, &action)
            .expect("encoded action must decode back to original");
    }

    #[test]
    fn schema_shift_roundtrip(state in 1u16..0x8000u16) {
        let action = Action::Shift(StateId(state));
        let encoded = validate_action_encoding(&action).unwrap();
        prop_assert_eq!(encoded, state);
        validate_action_decoding(encoded, &action).unwrap();
    }

    #[test]
    fn schema_reduce_roundtrip(rule in 0u16..0x7FFFu16) {
        let action = Action::Reduce(RuleId(rule));
        let encoded = validate_action_encoding(&action).unwrap();
        prop_assert!(encoded & 0x8000 != 0);
        prop_assert_eq!(encoded & 0x7FFF, rule);
        validate_action_decoding(encoded, &action).unwrap();
    }
}

// ---------------------------------------------------------------------------
// 2. Table compression: action and goto tables
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn action_table_compression_preserves_structure(
        n_states in 1usize..8,
        n_symbols in 1usize..8,
        seed in any::<u64>(),
    ) {
        use std::collections::BTreeMap;

        let mut rng = seed;
        let mut action_table: Vec<Vec<Vec<Action>>> = Vec::new();
        for _ in 0..n_states {
            let mut row = Vec::new();
            for _ in 0..n_symbols {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                let choice = (rng >> 60) & 0x3;
                let cell = match choice {
                    0 => vec![],
                    1 => vec![Action::Shift(StateId(((rng >> 48) as u16 % 100) + 1))],
                    2 => vec![Action::Reduce(RuleId((rng >> 32) as u16 % 50))],
                    _ => vec![Action::Accept],
                };
                row.push(cell);
            }
            action_table.push(row);
        }

        let symbol_to_index: BTreeMap<adze_ir::SymbolId, usize> = (0..n_symbols)
            .map(|i| (adze_ir::SymbolId(i as u16), i))
            .collect();

        let compressor = TableCompressor::new();
        let compressed = compressor
            .compress_action_table_small(&action_table, &symbol_to_index)
            .expect("compression must succeed");

        prop_assert_eq!(compressed.row_offsets.len(), n_states + 1);

        for window in compressed.row_offsets.windows(2) {
            prop_assert!(window[0] <= window[1],
                "row offsets must be non-decreasing: {} > {}", window[0], window[1]);
        }

        prop_assert_eq!(compressed.default_actions.len(), n_states);

        prop_assert_eq!(
            *compressed.row_offsets.last().unwrap() as usize,
            compressed.data.len()
        );
    }

    #[test]
    fn goto_table_compression_preserves_structure(
        n_states in 1usize..8,
        n_symbols in 1usize..8,
        seed in any::<u64>(),
    ) {
        let mut rng = seed;
        let mut goto_table: Vec<Vec<StateId>> = Vec::new();
        for _ in 0..n_states {
            let mut row = Vec::new();
            for _ in 0..n_symbols {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                row.push(StateId((rng >> 48) as u16 % 20));
            }
            goto_table.push(row);
        }

        let compressor = TableCompressor::new();
        let compressed = compressor
            .compress_goto_table_small(&goto_table)
            .expect("goto compression must succeed");

        prop_assert_eq!(compressed.row_offsets.len(), n_states + 1);

        for window in compressed.row_offsets.windows(2) {
            prop_assert!(window[0] <= window[1]);
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Symbol metadata flags: bitwise operations must be reversible
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_metadata_flags_roundtrip(
        visible in any::<bool>(),
        named in any::<bool>(),
        hidden in any::<bool>(),
        auxiliary in any::<bool>(),
        supertype in any::<bool>(),
    ) {
        let metadata = create_symbol_metadata(visible, named, hidden, auxiliary, supertype);

        prop_assert_eq!((metadata & symbol_metadata::VISIBLE) != 0, visible);
        prop_assert_eq!((metadata & symbol_metadata::NAMED) != 0, named);
        prop_assert_eq!((metadata & symbol_metadata::HIDDEN) != 0, hidden);
        prop_assert_eq!((metadata & symbol_metadata::AUXILIARY) != 0, auxiliary);
        prop_assert_eq!((metadata & symbol_metadata::SUPERTYPE) != 0, supertype);

        let all_flags = symbol_metadata::VISIBLE
            | symbol_metadata::NAMED
            | symbol_metadata::HIDDEN
            | symbol_metadata::AUXILIARY
            | symbol_metadata::SUPERTYPE;
        prop_assert_eq!(metadata & !all_flags, 0, "no undefined bits should be set");
    }

    #[test]
    fn symbol_metadata_individual_flags(flag_index in 0u8..5) {
        let mut flags = [false; 5];
        flags[flag_index as usize] = true;
        let metadata = create_symbol_metadata(flags[0], flags[1], flags[2], flags[3], flags[4]);

        let expected = match flag_index {
            0 => symbol_metadata::VISIBLE,
            1 => symbol_metadata::NAMED,
            2 => symbol_metadata::HIDDEN,
            3 => symbol_metadata::AUXILIARY,
            4 => symbol_metadata::SUPERTYPE,
            _ => unreachable!(),
        };
        prop_assert_eq!(metadata, expected);
        prop_assert_eq!(metadata.count_ones(), 1);
    }
}

// ---------------------------------------------------------------------------
// 4. ABI version validation: reject invalid versions
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn invalid_abi_version_rejected(version in any::<u32>().prop_filter(
        "must not be a valid ABI version",
        |v| *v != TREE_SITTER_LANGUAGE_VERSION,
    )) {
        let tables = adze_tablegen::CompressedParseTable::new_for_testing(10, 20);

        let lang = adze_tablegen::validation::TSLanguage {
            version,
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
            external_scanner_data: adze_tablegen::validation::TSExternalScannerData {
                states: std::ptr::null(),
                symbol_map: std::ptr::null(),
                create: None,
                destroy: None,
                scan: None,
                serialize: None,
                deserialize: None,
            },
            primary_state_ids: std::ptr::null(),
        };

        let validator = LanguageValidator::new(&lang, &tables);
        let result = validator.validate();

        prop_assert!(result.is_err());
        let errors = result.unwrap_err();
        let has_version_err = errors.iter().any(|e| matches!(
            e,
            ValidationError::InvalidVersion { expected: 15, actual } if *actual == version
        ));
        prop_assert!(has_version_err);
    }

    #[test]
    fn valid_abi_version_constant_in_range(
        version in TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION..=TREE_SITTER_LANGUAGE_VERSION,
    ) {
        prop_assert!(version <= TREE_SITTER_LANGUAGE_VERSION);
        prop_assert!(version >= TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION);
    }
}

// ---------------------------------------------------------------------------
// 5. Production ID map: encode/decode roundtrip via small-table encoding
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn small_table_reduce_encodes_1based_production_id(rule_id in 0u16..0x4000u16) {
        let action = Action::Reduce(RuleId(rule_id));
        let compressor = TableCompressor::new();
        let encoded = compressor
            .encode_action_small(&action)
            .expect("valid reduce must encode");

        // Encoding: 0x8000 | (rule_id + 1)  (1-based production ID)
        prop_assert_eq!(encoded, 0x8000 | (rule_id + 1));
        prop_assert!(encoded & 0x8000 != 0);

        // Decode back: subtract 1 from the low 15 bits to get the 0-based rule id
        let decoded_rule = (encoded & 0x7FFF) - 1;
        prop_assert_eq!(decoded_rule, rule_id);
    }

    #[test]
    fn small_table_shift_identity(state in 0u16..0x8000u16) {
        let action = Action::Shift(StateId(state));
        let compressor = TableCompressor::new();
        let encoded = compressor
            .encode_action_small(&action)
            .expect("valid shift must encode");

        prop_assert_eq!(encoded, state);
        prop_assert!(encoded & 0x8000 == 0);
    }

    #[test]
    fn small_table_special_actions_encode_correctly(action in small_table_action()) {
        let compressor = TableCompressor::new();
        let encoded = compressor
            .encode_action_small(&action)
            .expect("valid action must encode");

        match &action {
            Action::Accept => prop_assert_eq!(encoded, 0xFFFF),
            Action::Error => prop_assert_eq!(encoded, 0xFFFE),
            Action::Recover => prop_assert_eq!(encoded, 0xFFFD),
            Action::Shift(s) => prop_assert_eq!(encoded, s.0),
            Action::Reduce(r) => prop_assert_eq!(encoded, 0x8000 | (r.0 + 1)),
            _ => {}
        }
    }
}
