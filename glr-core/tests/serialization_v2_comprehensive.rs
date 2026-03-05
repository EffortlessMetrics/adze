#![cfg(feature = "test-api")]

//! Comprehensive v2 serialization tests for ParseTable.
//!
//! Run with:
//!   cargo test -p adze-glr-core --test serialization_v2_comprehensive --features test-api

#[cfg(feature = "serialization")]
mod tests {
    use adze_glr_core::serialization::{
        DeserializationError, PARSE_TABLE_FORMAT_VERSION, SerializationError,
    };
    use adze_glr_core::{
        Action, GotoIndexing, LexMode, ParseRule, ParseTable, StateId, SymbolId, SymbolMetadata,
    };
    use adze_ir::RuleId;

    // ---------------------------------------------------------------
    // Helpers
    // ---------------------------------------------------------------

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
            alias_sequences: vec![],
            field_names: vec![],
            field_map: Default::default(),
        }
    }

    /// Compare all serialized fields of two ParseTables.
    fn assert_tables_eq(a: &ParseTable, b: &ParseTable) {
        assert_eq!(a.state_count, b.state_count, "state_count");
        assert_eq!(a.symbol_count, b.symbol_count, "symbol_count");
        assert_eq!(a.action_table, b.action_table, "action_table");
        assert_eq!(a.goto_table, b.goto_table, "goto_table");
        assert_eq!(a.eof_symbol, b.eof_symbol, "eof_symbol");
        assert_eq!(a.start_symbol, b.start_symbol, "start_symbol");
        assert_eq!(a.initial_state, b.initial_state, "initial_state");
        assert_eq!(a.token_count, b.token_count, "token_count");
        assert_eq!(
            a.external_token_count, b.external_token_count,
            "external_token_count"
        );
        assert_eq!(a.goto_indexing, b.goto_indexing, "goto_indexing");
        assert_eq!(a.extras, b.extras, "extras");
        assert_eq!(
            a.dynamic_prec_by_rule, b.dynamic_prec_by_rule,
            "dynamic_prec_by_rule"
        );
        assert_eq!(
            a.rule_assoc_by_rule, b.rule_assoc_by_rule,
            "rule_assoc_by_rule"
        );
        assert_eq!(a.field_names, b.field_names, "field_names");
        assert_eq!(a.field_map, b.field_map, "field_map");
        assert_eq!(a.lex_modes, b.lex_modes, "lex_modes");
        assert_eq!(a.index_to_symbol, b.index_to_symbol, "index_to_symbol");
        assert_eq!(a.symbol_to_index, b.symbol_to_index, "symbol_to_index");
        assert_eq!(
            a.nonterminal_to_index, b.nonterminal_to_index,
            "nonterminal_to_index"
        );
        assert_eq!(a.alias_sequences, b.alias_sequences, "alias_sequences");
        assert_eq!(
            a.external_scanner_states, b.external_scanner_states,
            "external_scanner_states"
        );
        assert_eq!(a.rules.len(), b.rules.len(), "rules length");
        for (i, (ra, rb)) in a.rules.iter().zip(b.rules.iter()).enumerate() {
            assert_eq!(ra.lhs, rb.lhs, "rule[{i}].lhs");
            assert_eq!(ra.rhs_len, rb.rhs_len, "rule[{i}].rhs_len");
        }
        assert_eq!(
            a.symbol_metadata.len(),
            b.symbol_metadata.len(),
            "symbol_metadata length"
        );
        for (i, (ma, mb)) in a
            .symbol_metadata
            .iter()
            .zip(b.symbol_metadata.iter())
            .enumerate()
        {
            assert_eq!(ma.name, mb.name, "meta[{i}].name");
            assert_eq!(ma.is_visible, mb.is_visible, "meta[{i}].is_visible");
            assert_eq!(ma.is_named, mb.is_named, "meta[{i}].is_named");
            assert_eq!(ma.is_supertype, mb.is_supertype, "meta[{i}].is_supertype");
            assert_eq!(ma.is_terminal, mb.is_terminal, "meta[{i}].is_terminal");
            assert_eq!(ma.is_extra, mb.is_extra, "meta[{i}].is_extra");
            assert_eq!(ma.is_fragile, mb.is_fragile, "meta[{i}].is_fragile");
            assert_eq!(ma.symbol_id, mb.symbol_id, "meta[{i}].symbol_id");
        }
    }

    /// Build a `VersionedParseTable` with an arbitrary version for testing.
    fn craft_wrong_version(version: u32, valid_bytes: &[u8]) -> Vec<u8> {
        #[derive(serde::Serialize)]
        struct FakeVersioned {
            version: u32,
            data: Vec<u8>,
        }
        #[derive(serde::Deserialize)]
        struct FakeVersionedDe {
            #[allow(dead_code)]
            version: u32,
            data: Vec<u8>,
        }
        let original: FakeVersionedDe = postcard::from_bytes(valid_bytes).expect("decode original");
        let fake = FakeVersioned {
            version,
            data: original.data,
        };
        postcard::to_stdvec(&fake).expect("encode fake")
    }

    // ===================================================================
    // 1. Roundtrip serialization (10 tests)
    // ===================================================================

    #[test]
    fn roundtrip_default_table() {
        let table = ParseTable::default();
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_tables_eq(&table, &restored);
    }

    #[test]
    fn roundtrip_minimal_1x1() {
        let table = make_table(1, 1);
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_tables_eq(&table, &restored);
    }

    #[test]
    fn roundtrip_medium_table() {
        let table = make_table(10, 8);
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_tables_eq(&table, &restored);
    }

    #[test]
    fn roundtrip_preserves_all_action_variants() {
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
        assert_tables_eq(&table, &restored);
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
        assert_tables_eq(&table, &restored);
    }

    #[test]
    fn roundtrip_preserves_rules_and_fields() {
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
        assert_tables_eq(&table, &restored);
    }

    #[test]
    fn roundtrip_preserves_eof_and_start_symbols() {
        let mut table = make_table(2, 2);
        table.eof_symbol = SymbolId(99);
        table.start_symbol = SymbolId(42);
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(restored.eof_symbol, SymbolId(99));
        assert_eq!(restored.start_symbol, SymbolId(42));
    }

    #[test]
    fn roundtrip_preserves_extras() {
        let mut table = make_table(2, 4);
        table.extras = vec![SymbolId(2), SymbolId(3)];
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(restored.extras, [SymbolId(2), SymbolId(3)]);
    }

    #[test]
    fn roundtrip_preserves_dynamic_precedence_and_assoc() {
        let mut table = make_table(1, 1);
        table.dynamic_prec_by_rule = vec![0, -1, 2];
        table.rule_assoc_by_rule = vec![1, -1, 0];
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(restored.dynamic_prec_by_rule, [0, -1, 2]);
        assert_eq!(restored.rule_assoc_by_rule, [1, -1, 0]);
    }

    #[test]
    fn roundtrip_preserves_alias_sequences() {
        let mut table = make_table(1, 1);
        table.alias_sequences = vec![
            vec![None, Some(SymbolId(5))],
            vec![Some(SymbolId(3)), None, None],
        ];
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(restored.alias_sequences, table.alias_sequences);
    }

    // ===================================================================
    // 2. Serialized bytes are non-empty (5 tests)
    // ===================================================================

    #[test]
    fn bytes_nonempty_default_table() {
        let bytes = ParseTable::default().to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn bytes_nonempty_minimal_table() {
        let bytes = make_table(1, 1).to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn bytes_nonempty_table_with_actions() {
        let mut table = make_table(1, 2);
        table.action_table[0] = vec![vec![Action::Shift(StateId(0))], vec![Action::Accept]];
        let bytes = table.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn bytes_nonempty_table_with_metadata() {
        let mut table = make_table(1, 1);
        table.symbol_metadata = vec![SymbolMetadata {
            name: "tok".into(),
            is_visible: true,
            is_named: false,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(0),
        }];
        let bytes = table.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn bytes_nonempty_table_with_rules() {
        let mut table = make_table(1, 1);
        table.rules = vec![ParseRule {
            lhs: SymbolId(0),
            rhs_len: 1,
        }];
        let bytes = table.to_bytes().unwrap();
        assert!(!bytes.is_empty());
    }

    // ===================================================================
    // 3. Deserialization of corrupted bytes fails (8 tests)
    // ===================================================================

    #[test]
    fn corrupt_empty_bytes() {
        assert!(ParseTable::from_bytes(&[]).is_err());
    }

    #[test]
    fn corrupt_random_bytes() {
        let garbage: Vec<u8> = (0..128).map(|i| (i * 37 + 13) as u8).collect();
        assert!(ParseTable::from_bytes(&garbage).is_err());
    }

    #[test]
    fn corrupt_truncated_half() {
        let bytes = make_table(4, 4).to_bytes().unwrap();
        let half = bytes.len() / 2;
        assert!(ParseTable::from_bytes(&bytes[..half]).is_err());
    }

    #[test]
    fn corrupt_truncated_single_byte() {
        let bytes = make_table(2, 2).to_bytes().unwrap();
        assert!(ParseTable::from_bytes(&bytes[..1]).is_err());
    }

    #[test]
    fn corrupt_single_zero_byte() {
        assert!(ParseTable::from_bytes(&[0]).is_err());
    }

    #[test]
    fn corrupt_wrong_version() {
        let bytes = make_table(2, 2).to_bytes().unwrap();
        let bad = craft_wrong_version(999, &bytes);
        let err = ParseTable::from_bytes(&bad).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Incompatible format version"), "got: {msg}");
    }

    #[test]
    fn corrupt_wrong_version_zero() {
        let bytes = make_table(1, 1).to_bytes().unwrap();
        let bad = craft_wrong_version(0, &bytes);
        assert!(ParseTable::from_bytes(&bad).is_err());
    }

    #[test]
    fn corrupt_flipped_bits_in_payload() {
        let mut bytes = make_table(3, 3).to_bytes().unwrap();
        // Flip bits in the middle of the payload
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        bytes[mid + 1] ^= 0xFF;
        // May decode to garbage or fail — either way must not panic.
        let _ = ParseTable::from_bytes(&bytes);
    }

    // ===================================================================
    // 4. Determinism (5 tests)
    // ===================================================================

    #[test]
    fn determinism_basic() {
        let table = make_table(3, 4);
        let a = table.to_bytes().unwrap();
        let b = table.to_bytes().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn determinism_double_roundtrip() {
        let table = make_table(5, 3);
        let bytes1 = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes1).unwrap();
        let bytes2 = restored.to_bytes().unwrap();
        assert_eq!(bytes1, bytes2);
    }

    #[test]
    fn determinism_triple_roundtrip() {
        let table = make_table(4, 4);
        let b1 = table.to_bytes().unwrap();
        let t2 = ParseTable::from_bytes(&b1).unwrap();
        let b2 = t2.to_bytes().unwrap();
        let t3 = ParseTable::from_bytes(&b2).unwrap();
        let b3 = t3.to_bytes().unwrap();
        assert_eq!(b1, b2);
        assert_eq!(b2, b3);
    }

    #[test]
    fn determinism_with_complex_actions() {
        let mut table = make_table(2, 3);
        table.action_table[0][0] = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
        table.action_table[1][2] = vec![Action::Fork(vec![
            Action::Shift(StateId(0)),
            Action::Reduce(RuleId(1)),
        ])];
        let a = table.to_bytes().unwrap();
        let b = table.to_bytes().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn determinism_with_metadata() {
        let mut table = make_table(1, 1);
        table.symbol_metadata = vec![
            SymbolMetadata {
                name: "a".into(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: false,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(0),
            },
            SymbolMetadata {
                name: "b".into(),
                is_visible: false,
                is_named: false,
                is_supertype: true,
                is_terminal: true,
                is_extra: true,
                is_fragile: true,
                symbol_id: SymbolId(1),
            },
        ];
        let a = table.to_bytes().unwrap();
        let b = table.to_bytes().unwrap();
        assert_eq!(a, b);
    }

    // ===================================================================
    // 5. Various grammar topologies (8 tests)
    // ===================================================================

    #[test]
    fn topology_linear_chain() {
        let mut table = make_table(5, 2);
        for s in 0..4 {
            table.action_table[s][0] = vec![Action::Shift(StateId((s + 1) as u16))];
        }
        table.action_table[4][1] = vec![Action::Accept];
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_tables_eq(&table, &restored);
    }

    #[test]
    fn topology_wide_fanout() {
        let table = make_table(2, 20);
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_tables_eq(&table, &restored);
    }

    #[test]
    fn topology_deep_narrow() {
        let table = make_table(30, 2);
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_tables_eq(&table, &restored);
    }

    #[test]
    fn topology_multi_action_glr_cells() {
        let mut table = make_table(3, 3);
        table.action_table[0][0] = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
        table.action_table[1][1] = vec![
            Action::Shift(StateId(2)),
            Action::Reduce(RuleId(1)),
            Action::Reduce(RuleId(2)),
        ];
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_tables_eq(&table, &restored);
    }

    #[test]
    fn topology_many_rules() {
        let mut table = make_table(2, 2);
        table.rules = (0..50)
            .map(|i| ParseRule {
                lhs: SymbolId(i % 10),
                rhs_len: i % 5,
            })
            .collect();
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_tables_eq(&table, &restored);
    }

    #[test]
    fn topology_nonterminal_to_index_populated() {
        let mut table = make_table(3, 4);
        table.nonterminal_to_index.insert(SymbolId(10), 0);
        table.nonterminal_to_index.insert(SymbolId(11), 1);
        table.nonterminal_to_index.insert(SymbolId(12), 2);
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_tables_eq(&table, &restored);
    }

    #[test]
    fn topology_symbol_to_index_populated() {
        let mut table = make_table(2, 3);
        table.symbol_to_index.insert(SymbolId(0), 0);
        table.symbol_to_index.insert(SymbolId(1), 1);
        table.symbol_to_index.insert(SymbolId(2), 2);
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_tables_eq(&table, &restored);
    }

    #[test]
    fn topology_direct_symbol_id_goto() {
        let table = make_table(2, 3).remap_goto_to_direct_symbol_id();
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(restored.goto_indexing, GotoIndexing::DirectSymbolId);
    }

    // ===================================================================
    // 6. Serialized size properties (5 tests)
    // ===================================================================

    #[test]
    fn size_larger_tables_produce_more_bytes() {
        let small = make_table(2, 2).to_bytes().unwrap();
        let large = make_table(20, 20).to_bytes().unwrap();
        assert!(
            large.len() > small.len(),
            "20x20 ({}) should be larger than 2x2 ({})",
            large.len(),
            small.len()
        );
    }

    #[test]
    fn size_grows_with_state_count() {
        let a = make_table(5, 3).to_bytes().unwrap();
        let b = make_table(15, 3).to_bytes().unwrap();
        assert!(b.len() > a.len());
    }

    #[test]
    fn size_grows_with_symbol_count() {
        let a = make_table(3, 5).to_bytes().unwrap();
        let b = make_table(3, 15).to_bytes().unwrap();
        assert!(b.len() > a.len());
    }

    #[test]
    fn size_small_table_under_limit() {
        let bytes = make_table(2, 2).to_bytes().unwrap();
        assert!(
            bytes.len() < 10_000,
            "small table should be under 10KB, got {}",
            bytes.len()
        );
    }

    #[test]
    fn size_includes_version_overhead() {
        // Bytes must be at least a few bytes for the version wrapper.
        let bytes = ParseTable::default().to_bytes().unwrap();
        assert!(bytes.len() >= 4, "version wrapper must add some overhead");
    }

    // ===================================================================
    // 7. Multiple roundtrips (5 tests)
    // ===================================================================

    #[test]
    fn multi_roundtrip_two() {
        let table = make_table(3, 3);
        let b1 = table.to_bytes().unwrap();
        let t1 = ParseTable::from_bytes(&b1).unwrap();
        let b2 = t1.to_bytes().unwrap();
        assert_eq!(b1, b2);
    }

    #[test]
    fn multi_roundtrip_three() {
        let table = make_table(4, 5);
        let mut bytes = table.to_bytes().unwrap();
        for _ in 0..3 {
            let t = ParseTable::from_bytes(&bytes).unwrap();
            let new_bytes = t.to_bytes().unwrap();
            assert_eq!(bytes, new_bytes);
            bytes = new_bytes;
        }
    }

    #[test]
    fn multi_roundtrip_five() {
        let mut table = make_table(2, 3);
        table.extras = vec![SymbolId(1)];
        table.rules = vec![ParseRule {
            lhs: SymbolId(0),
            rhs_len: 2,
        }];
        let original = table.to_bytes().unwrap();
        let mut bytes = original.clone();
        for _ in 0..5 {
            let t = ParseTable::from_bytes(&bytes).unwrap();
            bytes = t.to_bytes().unwrap();
        }
        assert_eq!(original, bytes);
    }

    #[test]
    fn multi_roundtrip_preserves_identity_each_step() {
        let table = make_table(3, 4);
        let b0 = table.to_bytes().unwrap();
        let t1 = ParseTable::from_bytes(&b0).unwrap();
        assert_tables_eq(&table, &t1);
        let t2 = ParseTable::from_bytes(&t1.to_bytes().unwrap()).unwrap();
        assert_tables_eq(&table, &t2);
    }

    #[test]
    fn multi_roundtrip_complex_table() {
        let mut table = make_table(3, 4);
        table.action_table[0][0] = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))];
        table.symbol_metadata = vec![SymbolMetadata {
            name: "expr".into(),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(0),
        }];
        table.rules = vec![ParseRule {
            lhs: SymbolId(0),
            rhs_len: 2,
        }];
        table.field_names = vec!["operand".into()];
        table.field_map.insert((RuleId(0), 0), 0);

        let original = table.to_bytes().unwrap();
        let mut bytes = original.clone();
        for _ in 0..4 {
            let t = ParseTable::from_bytes(&bytes).unwrap();
            bytes = t.to_bytes().unwrap();
        }
        assert_eq!(original, bytes);
    }

    // ===================================================================
    // 8. Edge cases (9 tests)
    // ===================================================================

    #[test]
    fn edge_max_u16_symbol_ids() {
        let mut table = make_table(1, 1);
        table.eof_symbol = SymbolId(u16::MAX);
        table.start_symbol = SymbolId(u16::MAX - 1);
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(restored.eof_symbol, SymbolId(u16::MAX));
        assert_eq!(restored.start_symbol, SymbolId(u16::MAX - 1));
    }

    #[test]
    fn edge_zero_state_table() {
        let table = make_table(0, 0);
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(restored.state_count, 0);
        assert_eq!(restored.symbol_count, 0);
    }

    #[test]
    fn edge_large_goto_table() {
        let mut table = make_table(10, 10);
        for s in 0..10 {
            for sym in 0..10 {
                table.goto_table[s][sym] = StateId((s * 10 + sym) as u16);
            }
        }
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_tables_eq(&table, &restored);
    }

    #[test]
    fn edge_empty_action_cells() {
        let mut table = make_table(2, 2);
        table.action_table[0][0] = vec![];
        table.action_table[1][1] = vec![];
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert!(restored.action_table[0][0].is_empty());
        assert!(restored.action_table[1][1].is_empty());
    }

    #[test]
    fn edge_nested_fork_actions() {
        let mut table = make_table(1, 1);
        table.action_table[0][0] = vec![Action::Fork(vec![
            Action::Fork(vec![Action::Shift(StateId(0))]),
            Action::Reduce(RuleId(0)),
        ])];
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(restored.action_table[0][0], table.action_table[0][0]);
    }

    #[test]
    fn edge_all_error_table() {
        let table = make_table(5, 5);
        // make_table already fills with Error
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        for row in &restored.action_table {
            for cell in row {
                assert_eq!(cell, &vec![Action::Error]);
            }
        }
    }

    #[test]
    fn edge_all_accept_table() {
        let mut table = make_table(3, 3);
        for row in &mut table.action_table {
            for cell in row {
                *cell = vec![Action::Accept];
            }
        }
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        for row in &restored.action_table {
            for cell in row {
                assert_eq!(cell, &vec![Action::Accept]);
            }
        }
    }

    #[test]
    fn edge_long_field_names() {
        let mut table = make_table(1, 1);
        table.field_names = (0..20).map(|i| format!("field_name_{i:04}")).collect();
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(restored.field_names.len(), 20);
        assert_eq!(restored.field_names[0], "field_name_0000");
        assert_eq!(restored.field_names[19], "field_name_0019");
    }

    #[test]
    fn edge_large_alias_sequences() {
        let mut table = make_table(1, 1);
        table.alias_sequences = (0..10)
            .map(|i| {
                (0..5)
                    .map(|j| {
                        if (i + j) % 2 == 0 {
                            Some(SymbolId((i * 5 + j) as u16))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .collect();
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(restored.alias_sequences, table.alias_sequences);
    }

    // ===================================================================
    // Bonus: error message quality (2 tests)
    // ===================================================================

    #[test]
    fn error_display_incompatible_version() {
        let err = DeserializationError::IncompatibleVersion {
            expected: PARSE_TABLE_FORMAT_VERSION,
            actual: 0,
        };
        let msg = err.to_string();
        assert!(msg.contains("expected"), "missing 'expected': {msg}");
        assert!(msg.contains("got"), "missing 'got': {msg}");
    }

    #[test]
    fn error_display_serialization_validation() {
        let err = SerializationError::ValidationFailed("bad data".into());
        assert!(err.to_string().contains("bad data"));
    }
}
