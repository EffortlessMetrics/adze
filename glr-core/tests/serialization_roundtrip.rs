#![cfg(feature = "test-api")]

//! Serialization round-trip integration tests for ParseTable.
//!
//! Run with: cargo test -p adze-glr-core --test serialization_roundtrip --features test-api

// The serialization module is gated on `feature = "serialization"`, so all tests
// are wrapped in a cfg block. The `test-api` gate above is the file-level guard.
#[cfg(feature = "serialization")]
mod tests {
    use adze_glr_core::serialization::*;
    use adze_glr_core::*;
    use adze_ir::RuleId;

    /// Build a minimal ParseTable suitable for round-trip testing.
    fn make_table(states: usize, symbols: usize) -> ParseTable {
        ParseTable {
            action_table: vec![vec![vec![Action::Error]; symbols]; states],
            goto_table: vec![vec![StateId(0); symbols]; states],
            symbol_metadata: vec![],
            state_count: states,
            symbol_count: symbols,
            symbol_to_index: Default::default(),
            index_to_symbol: (0..symbols).map(|i| SymbolId(i as u16)).collect(),
            external_scanner_states: vec![vec![]; states],
            rules: vec![],
            nonterminal_to_index: Default::default(),
            goto_indexing: GotoIndexing::NonterminalMap,
            eof_symbol: SymbolId(0),
            start_symbol: SymbolId(1),
            grammar: Default::default(),
            initial_state: StateId(0),
            token_count: symbols,
            external_token_count: 0,
            lex_modes: vec![
                LexMode {
                    lex_state: 0,
                    external_lex_state: 0,
                };
                states
            ],
            extras: vec![],
            dynamic_prec_by_rule: vec![],
            rule_assoc_by_rule: vec![],
            field_names: vec![],
            field_map: Default::default(),
            alias_sequences: vec![],
        }
    }

    // ---------------------------------------------------------------
    // 1. SerializationFormat variants and version constant
    // ---------------------------------------------------------------

    #[test]
    fn version_constant_is_positive() {
        assert!(PARSE_TABLE_FORMAT_VERSION > 0, "version must be > 0");
    }

    #[test]
    fn version_constant_is_at_least_v2() {
        // The module header documents v2 as the current postcard-based format.
        assert!(
            PARSE_TABLE_FORMAT_VERSION >= 2,
            "current format should be >= 2 (postcard era)"
        );
    }

    // ---------------------------------------------------------------
    // 2. SerializationError Display variants
    // ---------------------------------------------------------------

    #[test]
    fn serialization_error_encoding_failed_message() {
        // Trigger a real postcard error by abusing the From impl.
        let postcard_err = postcard::Error::SerializeBufferFull;
        let err = SerializationError::from(postcard_err);
        let msg = err.to_string();
        assert!(
            msg.contains("Postcard encoding failed"),
            "unexpected message: {msg}"
        );
    }

    // ---------------------------------------------------------------
    // 3. DeserializationError Display variants
    // ---------------------------------------------------------------

    #[test]
    fn deserialization_error_validation_failed_message() {
        let err = DeserializationError::ValidationFailed("bad field XYZ".into());
        let msg = err.to_string();
        assert!(msg.contains("validation failed"), "unexpected: {msg}");
        assert!(msg.contains("bad field XYZ"), "missing detail: {msg}");
    }

    #[test]
    fn deserialization_error_decoding_failed_message() {
        let postcard_err = postcard::Error::DeserializeUnexpectedEnd;
        let err = DeserializationError::from(postcard_err);
        let msg = err.to_string();
        assert!(
            msg.contains("Postcard decoding failed"),
            "unexpected: {msg}"
        );
    }

    // ---------------------------------------------------------------
    // 4. Round-trip tests
    // ---------------------------------------------------------------

    #[test]
    fn roundtrip_preserves_state_and_symbol_counts() {
        let table = make_table(5, 3);
        let bytes = table.to_bytes().expect("serialize");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize");
        assert_eq!(restored.state_count, 5);
        assert_eq!(restored.symbol_count, 3);
    }

    #[test]
    fn roundtrip_preserves_eof_and_start_symbols() {
        let mut table = make_table(2, 4);
        table.eof_symbol = SymbolId(99);
        table.start_symbol = SymbolId(42);
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(restored.eof_symbol, SymbolId(99));
        assert_eq!(restored.start_symbol, SymbolId(42));
    }

    #[test]
    fn roundtrip_preserves_action_variants() {
        let mut table = make_table(1, 6);
        table.action_table[0] = vec![
            vec![Action::Shift(StateId(7))],
            vec![Action::Reduce(RuleId(3))],
            vec![Action::Accept],
            vec![Action::Error],
            vec![Action::Recover],
            vec![Action::Fork(vec![
                Action::Shift(StateId(1)),
                Action::Reduce(RuleId(0)),
            ])],
        ];
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();

        assert_eq!(restored.action_table[0][0], vec![Action::Shift(StateId(7))]);
        assert_eq!(restored.action_table[0][1], vec![Action::Reduce(RuleId(3))]);
        assert_eq!(restored.action_table[0][2], vec![Action::Accept]);
        assert_eq!(restored.action_table[0][3], vec![Action::Error]);
        assert_eq!(restored.action_table[0][4], vec![Action::Recover]);
        assert_eq!(
            restored.action_table[0][5],
            vec![Action::Fork(vec![
                Action::Shift(StateId(1)),
                Action::Reduce(RuleId(0)),
            ])]
        );
    }

    #[test]
    fn roundtrip_preserves_goto_indexing_variant() {
        let mut table = make_table(1, 1);
        table.goto_indexing = GotoIndexing::DirectSymbolId;
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(restored.goto_indexing, GotoIndexing::DirectSymbolId);
    }

    #[test]
    fn roundtrip_preserves_symbol_metadata() {
        let mut table = make_table(1, 1);
        table.symbol_metadata = vec![SymbolMetadata {
            name: "identifier".into(),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: true,
            symbol_id: SymbolId(7),
        }];
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(restored.symbol_metadata.len(), 1);
        let m = &restored.symbol_metadata[0];
        assert_eq!(m.name, "identifier");
        assert!(m.is_visible);
        assert!(m.is_named);
        assert!(!m.is_supertype);
        assert!(m.is_terminal);
        assert!(!m.is_extra);
        assert!(m.is_fragile);
        assert_eq!(m.symbol_id, SymbolId(7));
    }

    #[test]
    fn roundtrip_preserves_rules_and_field_map() {
        let mut table = make_table(1, 1);
        table.rules = vec![
            ParseRule {
                lhs: SymbolId(10),
                rhs_len: 3,
            },
            ParseRule {
                lhs: SymbolId(20),
                rhs_len: 1,
            },
        ];
        table.field_names = vec!["left".into(), "right".into()];
        table.field_map.insert((RuleId(0), 0), 1);
        table.field_map.insert((RuleId(0), 2), 2);

        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();

        assert_eq!(restored.rules.len(), 2);
        assert_eq!(restored.rules[0].lhs, SymbolId(10));
        assert_eq!(restored.rules[0].rhs_len, 3);
        assert_eq!(restored.rules[1].lhs, SymbolId(20));
        assert_eq!(restored.rules[1].rhs_len, 1);
        assert_eq!(restored.field_names, vec!["left", "right"]);
        assert_eq!(restored.field_map.get(&(RuleId(0), 0)), Some(&1));
        assert_eq!(restored.field_map.get(&(RuleId(0), 2)), Some(&2));
    }

    #[test]
    fn roundtrip_deterministic_output() {
        let table = make_table(3, 4);
        let bytes1 = table.to_bytes().unwrap();
        let bytes2 = table.to_bytes().unwrap();
        assert_eq!(bytes1, bytes2, "serialization must be deterministic");
    }

    // ---------------------------------------------------------------
    // 5. Version compatibility checks
    // ---------------------------------------------------------------

    #[test]
    fn incompatible_version_returns_error() {
        let table = make_table(1, 1);
        let mut bytes = table.to_bytes().unwrap();

        // Corrupt the version field: postcard uses varint encoding at the
        // beginning of the outer VersionedParseTable. The first byte(s)
        // encode the version. Replacing the entire payload with a hand-crafted
        // VersionedParseTable that carries a wrong version is more reliable.
        let wrong_version_table = WrongVersionHelper::build(999, &bytes);
        let result = ParseTable::from_bytes(&wrong_version_table);

        assert!(result.is_err(), "wrong version must be rejected");
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("Incompatible format version"),
            "unexpected: {msg}"
        );
    }

    #[test]
    fn empty_bytes_return_error() {
        let result = ParseTable::from_bytes(&[]);
        assert!(result.is_err(), "empty input must fail");
    }

    #[test]
    fn truncated_bytes_return_error() {
        let table = make_table(2, 2);
        let bytes = table.to_bytes().unwrap();
        // Truncate to half the payload
        let half = bytes.len() / 2;
        let result = ParseTable::from_bytes(&bytes[..half]);
        assert!(result.is_err(), "truncated input must fail");
    }

    // ---------------------------------------------------------------
    // Helper: craft a VersionedParseTable with an arbitrary version
    // ---------------------------------------------------------------
    struct WrongVersionHelper;

    impl WrongVersionHelper {
        /// Build a postcard-encoded VersionedParseTable with a custom version
        /// but real inner data (taken from a valid serialization).
        fn build(version: u32, valid_bytes: &[u8]) -> Vec<u8> {
            // We need serde + postcard to build the wrapper.  Re-use the
            // private VersionedParseTable layout: { version: u32, data: Vec<u8> }
            // Since VersionedParseTable is private, we replicate its shape.
            #[derive(serde::Serialize)]
            struct FakeVersioned {
                version: u32,
                data: Vec<u8>,
            }

            // Extract the inner `data` bytes from the original serialization.
            // The original bytes = postcard(VersionedParseTable { version, data }).
            // We decode the original to get `data`, then re-encode with wrong version.
            #[derive(serde::Deserialize)]
            struct FakeVersionedDe {
                #[allow(dead_code)]
                version: u32,
                data: Vec<u8>,
            }

            let original: FakeVersionedDe =
                postcard::from_bytes(valid_bytes).expect("should decode original");

            let fake = FakeVersioned {
                version,
                data: original.data,
            };
            postcard::to_stdvec(&fake).expect("should encode fake")
        }
    }
}
